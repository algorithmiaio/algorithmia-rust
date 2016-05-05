//! Algorithm module for executing Algorithmia algorithms
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Algorithmia;
//! use algorithmia::algo::{Algorithm, Version};
//!
//! // Initialize with an API key
//! let client = Algorithmia::client("111112222233333444445555566");
//! let moving_avg = client.algo(("timeseries/SimpleMovingAverage", "0.1"));
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = (vec![0,1,2,3,15,4,5,6,7], 3);
//! let result: Vec<f64> = moving_avg.pipe(&input, None).unwrap().decode().unwrap();
//! println!("Completed with result: {:?}", result);
//! ```

use client::{Body, HttpClient};
use error::{Error, ApiErrorResponse};
use super::version::Version;

use rustc_serialize::{json, Decodable, Encodable};
use rustc_serialize::json::Json;
use rustc_serialize::base64::FromBase64;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::{self, Url};
use hyper::client::response::Response;

use std::borrow::Cow;
use std::io::{self, Read, Write};
use std::str::FromStr;
use std::{self, fmt};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

static ALGORITHM_BASE_PATH: &'static str = "v1/algo";

/// Types that represent either input to or output from an algorithm
pub enum AlgoIo {
    /// Input or output data that represents plain text (`text/plain`)
    Text(String),
    /// Input or output text that represents JSON (`application/json`)
    Json(Json),
    /// Input or output text that represents binary data (`application/octet-stream`)
    Binary(Vec<u8>),
}

/// Algorithmia algorithm - intialized from the `Algorithmia` builder
pub struct Algorithm {
    pub path: String,
    client: HttpClient,
}

/// Options used to alter the algorithm call, e.g. configuring the timeout
pub struct AlgoOptions {
    opts: HashMap<String, String>
}

pub struct AlgoRef {
    pub path: String
}

/// Metadata returned from the API
#[derive(RustcDecodable, Debug)]
pub struct AlgoMetadata {
    pub duration: f32,
    pub stdout: Option<String>,
    pub alerts: Option<Vec<String>>,
    pub content_type: String,
}

/// Successful API response that wraps the AlgoIo and its Metadata
pub struct AlgoResponse {
    pub metadata: AlgoMetadata,
    pub result: AlgoIo,
}

/// Alternate implementation for `EntryPoint`
///   that automatically decodes JSON input to the associate type.
///
/// # Examples
/// ```no_run
/// # use algorithmia::algo::*;
/// # #[derive(Default)]
/// # struct Algo;
/// impl DecodedEntryPoint for Algo {
///     // Expect input to be an array of 2 strings
///     type Input = (String, String);
///     fn apply_decoded(&self, input: Self::Input) -> Result<AlgoIo, Box<std::error::Error>> {
///         let msg = format!("{} - {}", input.0, input.1);
///         Ok(msg.into())
///     }
/// }
/// ```
pub trait DecodedEntryPoint: Default {
    type Input: Decodable;

    /// This method is an apply variant that will receive the decoded form of JSON input.
    ///   If decoding failed, a `DecoderError` will be returned before this method is invoked.
    #[allow(unused_variables)]
    fn apply_decoded(&self, input: Self::Input) -> Result<AlgoIo, Box<std::error::Error>>;

}

impl<T> EntryPoint for T where T: DecodedEntryPoint {
    fn apply(&self, input: AlgoIo) -> Result<AlgoIo, Box<std::error::Error>> {
        match input.as_json() {
            Some(obj) => {
                let encoded = try!(json::encode(&obj));
                let decoded = try!(json::decode(&encoded));
                self.apply_decoded(decoded)
            },
            None => Err(Error::UnsupportedInput.into()),
        }
    }
}

/// Implementing an algorithm involves overriding at least one of these methods
pub trait EntryPoint: Default {
    #[allow(unused_variables)]
    fn apply_str(&self, name: &str) -> Result<AlgoIo, Box<std::error::Error>> {
        Err(Error::UnsupportedInput.into())
    }
    #[allow(unused_variables)]
    fn apply_json(&self, json: &Json) -> Result<AlgoIo, Box<std::error::Error>> {
        Err(Error::UnsupportedInput.into())
    }
    #[allow(unused_variables)]
    fn apply_bytes(&self, bytes: &[u8]) -> Result<AlgoIo, Box<std::error::Error>> {
        Err(Error::UnsupportedInput.into())
    }

    /// The default implementation of this method calls
    /// `apply_str`, `apply_json`, or `apply_bytes` based on the input type.
    ///
    ///   - `AlgoIo::Text` results in call to  `apply_str`
    ///   - `AlgoIo::Json` results in call to  `apply_json`
    ///   - `AlgoIo::Binary` results in call to  `apply_bytes`
    ///
    /// If that call returns an `UnsupportedInput` error, then this method
    ///   method will may attempt to coerce the input into another type
    ///   and attempt one more call:
    ///
    ///   - `AlgoIo::Text` input will be JSON-encoded to call `apply_json`
    ///   - `AlgoIo::Json` input will be parse to see it can call `apply_str`
    fn apply(&self, input: AlgoIo) -> Result<AlgoIo, Box<std::error::Error>> {
        match &input {
            &AlgoIo::Text(ref text) => match self.apply_str(&text) {
                Err(err) => match err.downcast::<Error>().map(|b| *b) {
                    Ok(Error::UnsupportedInput) =>  match input.as_json() {
                        Some(json) => self.apply_json(&json),
                        None => Err(Error::UnsupportedInput.into()),
                    },
                    Ok(err) => Err(err.into()),
                    Err(err) => Err(err.into()),
                },
                ret => ret,
            },
            &AlgoIo::Json(ref json) => match self.apply_json(&json) {
                Err(err) => match err.downcast::<Error>().map(|b| *b) {
                    Ok(Error::UnsupportedInput) =>  match input.as_string() {
                        Some(text) => self.apply_str(&text),
                        None => Err(Error::UnsupportedInput.into()).into(),
                    },
                    Ok(err) => Err(err.into()),
                    Err(err) => Err(err.into()),
                },
                ret => ret,
            },
            &AlgoIo::Binary(ref bytes) => self.apply_bytes(bytes),
        }
    }
}

impl Algorithm {
    pub fn new(client: HttpClient, algo_ref: AlgoRef) -> Algorithm {
        let path: String = match algo_ref.path {
            ref p if p.starts_with("algo://") => p[7..].into(),
            ref p if p.starts_with("/") => p[1..].into(),
            p => p,
        };
        Algorithm {
            client: client,
            path: path,
        }
    }

    /// Get the API Endpoint URL for this Algorithm
    pub fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}", self.client.base_url, ALGORITHM_BASE_PATH, self.path);
        Url::parse(&url_string).unwrap()
    }

    /// Get the Algorithmia algo URI for this Algorithm
    pub fn to_algo_uri(&self) -> String {
        format!("algo://{}", self.path)
    }

    /// Execute an algorithm with
    ///
    /// Content-type is determined by the type of input_data
    ///   String => plain/text
    ///   Encodable => application/json
    ///   Byte slice => application/octet-stream
    ///
    /// To create encodable objects for complex input,
    ///     use `#[derive(RustcEncodable)]` on your struct
    ///
    /// If you want a string to be sent as application/json,
    ///    use `pipe_json(...)` instead
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::algo::Algorithm;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let moving_avg = client.algo("timeseries/SimpleMovingAverage/0.1");
    /// let input = (vec![0,1,2,3,15,4,5,6,7], 3);
    /// match moving_avg.pipe(&input, None) {
    ///     Ok(response) => println!("{}", response.as_json().unwrap()),
    ///     Err(err) => println!("ERROR: {}", err),
    /// };
    /// ```
    pub fn pipe<'a, I>(&'a self, input_data: I, options: Option<&'a AlgoOptions>)
                       -> Result<AlgoResponse, Error> where
        I: Into<AlgoIo>
    {
        let mut res = try!(match input_data.into() {
            AlgoIo::Text(text) => self.pipe_as(&*text, Mime(TopLevel::Text, SubLevel::Plain, vec![]), options),
            AlgoIo::Json(json) => {
                let encoded = try!(json::encode(&json));
                self.pipe_as(&*encoded, Mime(TopLevel::Application, SubLevel::Json, vec![]), options)
            },
            AlgoIo::Binary(bytes) => self.pipe_as(&*bytes, Mime(TopLevel::Application, SubLevel::Ext("octet-stream".into()), vec![]), options),
        });

        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));
        res_json.parse()
    }

    /// Execute an algorithm with explicitly set content-type
    ///
    ///
    /// `pipe` provides a JSON encoding/decoding wrapper around this method
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::algo::Algorithm;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let minmax  = client.algo("codeb34v3r/FindMinMax/0.1");
    ///
    /// let output = match minmax.pipe_json("[2,3,4]", None) {
    ///    Ok(response) => response.as_json().unwrap(),
    ///    Err(err) => panic!("{}", err),
    /// };
    pub fn pipe_json(&self, json_input: &str, options: Option<&AlgoOptions>) -> Result<AlgoResponse, Error> {
        let mut res = try!(self.pipe_as(json_input, Mime(TopLevel::Application, SubLevel::Json, vec![]), options));

        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));
        res_json.parse()
    }


    pub fn pipe_as<'a, B>(&'a self, input_data: B, content_type: Mime, options: Option<&'a AlgoOptions>)
                          -> Result<Response, hyper::error::Error>
        where B: Into<Body<'a>>
    {

        // Combine any existing paramaters with any
        let mut url = self.to_url();
        let original_params = url.query_pairs();
        let mut final_params: HashMap<&str, &str> = HashMap::new();

        if let Some(ref pairs) = original_params {
            for pair in pairs {
                final_params.insert(&*pair.0, &*pair.1);
            }
        }
        if let Some(ref opts) = options {
            for (k,v) in opts.iter() {
                final_params.insert(&*k, &*v);
            }
            // update query since AlgoOptions were provided
            url.set_query_from_pairs(final_params.iter().map(|(k,v)|(*k,*v)));
        }

        let req = self.client.post(url)
            .header(ContentType(content_type))
            .body(input_data);

        req.send()
    }
}

impl AlgoIo {
    /// If the `AlgoIo` is text (or a valid JSON string), returns the associated text
    pub fn as_string<'a>(&'a self) -> Option<&'a str> {
        match self {
            &AlgoIo::Text(ref text) => Some(text),
            &AlgoIo::Binary(_) => None,
            &AlgoIo::Json(Json::String(ref text)) => Some(text),
            _ => None,
        }
    }

    /// If the `AlgoIo` is Json (or text that can be JSON encoded), returns the associated JSON string
    pub fn as_json<'a>(&'a self) -> Option<Cow<'a, Json>> {
        match self {
            &AlgoIo::Text(ref text) => Some(Cow::Owned(Json::String(text.clone()))),
            &AlgoIo::Json(ref json) => Some(Cow::Borrowed(json)),
            &AlgoIo::Binary(_) => None,
        }
    }

    /// If the `AlgoIo` is binary, returns the associated byte slice
    pub fn as_bytes<'a>(&'a self) -> Option<&'a [u8]> {
        match self {
            &AlgoIo::Text(_) => None,
            &AlgoIo::Json(_) => None,
            &AlgoIo::Binary(ref bytes) => Some(bytes),
        }
    }

    /// If the `AlgoIo` is valid JSON, decode it to a particular type
    pub fn decode<D: Decodable>(&self) -> Result<D, Error> {
        let res_json = try!(self.as_json()
            .ok_or(Error::ContentTypeError("Input is not JSON".into())));
        let encoded = try!(json::encode(&res_json));
        json::decode::<D>(&encoded).map_err(|err| err.into())
    }
}

impl AlgoResponse {
    /// If the result is text (or a valid JSON string), returns the associated string
    pub fn as_string(self) -> Option<String> {
        match self.result {
            AlgoIo::Text(text) => Some(text),
            AlgoIo::Json(Json::String(text)) => Some(text),
            _ => None,
        }
    }

    /// If the result is Json (or text that can be JSON encoded), returns the associated JSON string
    pub fn as_json(self) -> Option<Json> {
        match self.result {
            AlgoIo::Json(json) => Some(json),
            AlgoIo::Text(text) => Some(Json::String(text)),
            _ => None,
        }
    }

    /// If the result is Binary, returns the associated byte slice
    pub fn as_bytes(self) -> Option<Vec<u8>> {
        match self.result {
            AlgoIo::Binary(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// If the result is valid JSON, decode it to a particular type
    pub fn decode<D: Decodable>(self) -> Result<D, Error> {
        let ct = self.metadata.content_type.clone();
        let res_json = try!(self.as_json()
            .ok_or(Error::ContentTypeError(ct)));
        let encoded = try!(json::encode(&res_json));
        json::decode::<D>(&encoded).map_err(|err| err.into())
    }
}

impl AlgoOptions {
    /// Initialize empty set of `AlgoOptions`
    pub fn new() -> AlgoOptions {
        AlgoOptions { opts: HashMap::new() }
    }

    /// Configure the timeout in seconds
    pub fn timeout(&mut self, timeout: u32) {
        self.opts.insert("timeout".into(), timeout.to_string());
    }

    /// Include algorithm stdout in the response metadata
    /// This has no affect unless authenticated as the owner of the algorithm
    pub fn stdout(&mut self, stdout: bool) {
        self.opts.insert("stdout".into(), stdout.to_string());
    }
}

impl FromStr for AlgoResponse {
    type Err = Error;
    fn from_str(json_str: &str) -> Result<Self, Self::Err> {
        // Early return if the response decodes into ApiErrorResponse
        if let Ok(err_res) = json::decode::<ApiErrorResponse>(json_str) {
            return Err(err_res.error.into())
        }

        // Parse into Json object
        let data = match Json::from_str(json_str) {
            Ok(d) => d,
            Err(err) => return Err(json::DecoderError::ParseError(err).into()),
        };

        // Construct the AlgoMetadata object
        let metadata = match data.search("metadata") {
            Some(meta_json) => match json::decode::<AlgoMetadata>(&meta_json.to_string()) {
                Ok(meta) => meta,
                Err(err) => return Err(err.into()),
            },
            None => return Err(json::DecoderError::MissingFieldError("metadata".into()).into()),
        };

        // Construct the AlgoIo object
        let result = match (&*metadata.content_type, data.search("result")) {
            ("void", _) => AlgoIo::Json(Json::Null),
            ("json", Some(json)) => AlgoIo::Json(json.clone()),
            ("text", Some(json)) => match json.as_string() {
                Some(text) => AlgoIo::Text(text.into()),
                None => return Err(Error::ContentTypeError("invalid text".into())),
            },
            ("binary", Some(json)) => match json.as_string() {
                Some(text) => AlgoIo::Binary(try!(text.from_base64())),
                None => return Err(Error::ContentTypeError("invalid text".into())),
            },
            (_, None) => return Err(json::DecoderError::MissingFieldError("result".into()).into()),
            (content_type, _) => return Err(Error::ContentTypeError(content_type.into())),
        };

        // Construct the AlgoResponse object
        Ok(AlgoResponse{ metadata: metadata, result: result})
    }
}

impl fmt::Display for AlgoResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.result {
            AlgoIo::Text(ref s) => f.write_str(s),
            AlgoIo::Json(ref s) => f.write_str(&s.to_string()),
            AlgoIo::Binary(ref bytes) => f.write_str(&String::from_utf8_lossy(bytes)),
        }
    }
}

impl Read for AlgoResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut out = buf; // why do I need this binding?
        match self.result {
            AlgoIo::Text(ref s) => out.write(s.as_bytes()),
            AlgoIo::Json(ref s) => out.write(s.to_string().as_bytes()),
            AlgoIo::Binary(ref bytes) => out.write(bytes),
        }
    }
}

impl <'a> From<&'a str> for AlgoRef {
    fn from(path: &'a str) -> Self {
        AlgoRef{ path:path.into() }
    }
}

impl <'a, V: Into<Version>> From<(&'a str, V)> for AlgoRef {
    fn from(path_parts: (&'a str, V)) -> Self {
        let (algo, version) = path_parts;
        let path = match version.into() {
            Version::Latest => format!("{}", algo),
            ref ver => format!("{}/{}", algo, ver),
        };

        AlgoRef{ path:path }
    }
}

impl <'a> From<&'a str> for AlgoIo {
    fn from(text: &'a str) -> Self {
        AlgoIo::Text(text.to_owned())
    }
}

impl From<String> for AlgoIo {
    fn from(text: String) -> Self {
        AlgoIo::Text(text)
    }
}

impl <'a> From<&'a [u8]> for AlgoIo {
    fn from(bytes: &'a [u8]) -> Self {
        AlgoIo::Binary(bytes.to_owned())
    }
}

impl From<Vec<u8>> for AlgoIo {
    fn from(bytes: Vec<u8>) -> Self {
        AlgoIo::Binary(bytes)
    }
}

impl From<Json> for AlgoIo {
    fn from(json: Json) -> Self {
        AlgoIo::Json(json)
    }
}

impl From<()> for AlgoIo {
    fn from(_unit: ()) -> Self {
        AlgoIo::Json(Json::Null)
    }
}

impl Deref for AlgoOptions {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &HashMap<String, String> { &self.opts }
}

impl DerefMut for AlgoOptions {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.opts }
}

impl <'a, E: Encodable> From<&'a E> for AlgoIo {
    fn from(encodable: &'a E) -> Self {
        // TODO: remove unwrap - either find a way to Box the encodable object and let pipe() encode it
        //       or store a result and let pipe() do error handling
        let encoded = json::encode(&encodable).unwrap();
        AlgoIo::Json(Json::from_str(&encoded).unwrap())
    }
}


#[cfg(test)]
mod tests {
    use Algorithmia;
    use super::*;

    fn mock_client() -> Algorithmia { Algorithmia::client("") }

    #[test]
    fn test_algo_without_version_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo("/anowell/Pinky");
        assert_eq!(algorithm.to_url().serialize_path().unwrap(), "/v1/algo/anowell/Pinky");
    }

    #[test]
    fn test_algo_without_prefix_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo("anowell/Pinky/0.1.0");
        assert_eq!(algorithm.to_url().serialize_path().unwrap(), "/v1/algo/anowell/Pinky/0.1.0");
    }

    #[test]
    fn test_algo_with_prefix_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo("algo://anowell/Pinky/0.1");
        assert_eq!(algorithm.to_url().serialize_path().unwrap(), "/v1/algo/anowell/Pinky/0.1");
    }

    #[test]
    fn test_algo_typesafe_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo(("anowell/Pinky", "abcdef123456"));
        assert_eq!(algorithm.to_url().serialize_path().unwrap(), "/v1/algo/anowell/Pinky/abcdef123456");
    }


    #[test]
    fn test_json_decoding() {
        let json_output = r#"{"metadata":{"duration":0.46739511,"content_type":"json"},"result":[5,41]}"#;
        let expected_meta = AlgoMetadata { duration: 0.46739511f32, stdout: None, alerts: None, content_type: "json".into()};
        let expected_result = [5, 41];
        let decoded = json_output.parse::<AlgoResponse>().unwrap();
        assert_eq!(expected_meta.duration, decoded.metadata.duration);
        assert_eq!(expected_result, &*decoded.decode::<Vec<i32>>().unwrap());
    }
}
