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
//! let result: Vec<f64> = moving_avg.pipe(&input).unwrap().decode().unwrap();
//! println!("Completed with result: {:?}", result);
//! ```

use client::{Body, HttpClient};
use error::{Error, ApiErrorResponse};
use super::version::Version;

use serde_json::{self, Value, Error as JsonError};
use serde_json::value::ToJson;
use serde::{Deserialize, Serialize};
use base64;

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

/// Types that can be used as input to an algorithm
pub enum AlgoInput<'a> {
    /// Data that will be sent with `Content-Type: text/plain`
    Text(Cow<'a, str>),
    /// Data that will be sent with `Content-Type: application/octet-stream`
    Binary(Cow<'a, [u8]>),
    /// Data that will be sent with `Content-Type: application/json`
    Json(Cow<'a, Value>),
}

/// Types that can store the output of an algorithm
pub enum AlgoOutput {
    /// Representation of result when `metadata.content_type` is 'text'
    Text(String),
    /// Representation of result when `metadata.content_type` is 'json'
    Json(Value),
    /// Representation of result when `metadata.content_type` is 'binary'
    Binary(Vec<u8>),
}

/// Algorithmia algorithm - intialized from the `Algorithmia` builder
pub struct Algorithm {
    pub path: String,
    options: AlgoOptions,
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
#[derive(Deserialize, Debug)]
pub struct AlgoMetadata {
    pub duration: f32,
    pub stdout: Option<String>,
    pub alerts: Option<Vec<String>>,
    pub content_type: String,
}

/// Successful API response that wraps the AlgoOutput and its Metadata
pub struct AlgoResponse {
    pub metadata: AlgoMetadata,
    pub result: AlgoOutput,
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
///     fn apply_decoded(&self, input: Self::Input) -> Result<AlgoOutput, Box<std::error::Error>> {
///         let msg = format!("{} - {}", input.0, input.1);
///         Ok(msg.into())
///     }
/// }
/// ```
pub trait DecodedEntryPoint: Default {
    type Input: Deserialize;

    /// This method is an apply variant that will receive the decoded form of JSON input.
    ///   If decoding failed, a `DecoderError` will be returned before this method is invoked.
    #[allow(unused_variables)]
    fn apply_decoded(&self, input: Self::Input) -> Result<AlgoOutput, Box<std::error::Error>>;

}

impl<T> EntryPoint for T where T: DecodedEntryPoint {
    fn apply<'a>(&self, input: AlgoInput<'a>) -> Result<AlgoOutput, Box<std::error::Error>> {
        match input.as_json() {
            Some(obj) => {
                let decoded = try!(serde_json::from_value(obj.into_owned()));
                self.apply_decoded(decoded)
            },
            None => Err(Error::UnsupportedInput.into()),
        }
    }
}

/// Implementing an algorithm involves overriding at least one of these methods
pub trait EntryPoint: Default {
    #[allow(unused_variables)]
    fn apply_str(&self, name: &str) -> Result<AlgoOutput, Box<std::error::Error>> {
        Err(Error::UnsupportedInput.into())
    }
    #[allow(unused_variables)]
    fn apply_json(&self, json: &Value) -> Result<AlgoOutput, Box<std::error::Error>> {
        Err(Error::UnsupportedInput.into())
    }
    #[allow(unused_variables)]
    fn apply_bytes(&self, bytes: &[u8]) -> Result<AlgoOutput, Box<std::error::Error>> {
        Err(Error::UnsupportedInput.into())
    }

    /// The default implementation of this method calls
    /// `apply_str`, `apply_json`, or `apply_bytes` based on the input type.
    ///
    ///   - `AlgoInput::Text` results in call to  `apply_str`
    ///   - `AlgoInput::Json` results in call to  `apply_json`
    ///   - `AlgoInput::Binary` results in call to  `apply_bytes`
    ///
    /// If that call returns an `UnsupportedInput` error, then this method
    ///   method will may attempt to coerce the input into another type
    ///   and attempt one more call:
    ///
    ///   - `AlgoInput::Text` input will be JSON-encoded to call `apply_json`
    ///   - `AlgoInput::Json` input will be parse to see it can call `apply_str`
    fn apply<'a>(&self, input: AlgoInput<'a>) -> Result<AlgoOutput, Box<std::error::Error>> {
        match &input {
            &AlgoInput::Text(ref text) => match self.apply_str(&text) {
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
            &AlgoInput::Json(ref json) => match self.apply_json(&json) {
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
            &AlgoInput::Binary(ref bytes) => self.apply_bytes(bytes),
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
            options: AlgoOptions::new(),
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
    /// match moving_avg.pipe(&input) {
    ///     Ok(response) => println!("{}", response.as_json().unwrap()),
    ///     Err(err) => println!("ERROR: {}", err),
    /// };
    /// ```
    pub fn pipe<'a, I>(&'a self, input_data: I) -> Result<AlgoResponse, Error>
        where I: Into<AlgoInput<'a>>
    {
        let mut res = try!(match input_data.into() {
            AlgoInput::Text(text) => self.pipe_as(&*text, Mime(TopLevel::Text, SubLevel::Plain, vec![])),
            AlgoInput::Json(json) => {
                let encoded = try!(serde_json::to_vec(&json));
                self.pipe_as(&*encoded, Mime(TopLevel::Application, SubLevel::Json, vec![]))
            },
            AlgoInput::Binary(bytes) => self.pipe_as(&*bytes, Mime(TopLevel::Application, SubLevel::Ext("octet-stream".into()), vec![])),
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
    /// let output = match minmax.pipe_json("[2,3,4]") {
    ///    Ok(response) => response.as_json().unwrap(),
    ///    Err(err) => panic!("{}", err),
    /// };
    pub fn pipe_json(&self, json_input: &str) -> Result<AlgoResponse, Error> {
        let mut res = try!(self.pipe_as(json_input, Mime(TopLevel::Application, SubLevel::Json, vec![])));

        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));
        res_json.parse()
    }


    pub fn pipe_as<'a, B>(&'a self, input_data: B, content_type: Mime)
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

        if !self.options.is_empty() {
            for (k,v) in self.options.iter() {
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

    /// Builder method to explicitly configure options
    pub fn set_options(&mut self, options: AlgoOptions) -> &mut Algorithm {
        self.options = options;
        self
    }

    /// Builder method to configure the timeout in seconds
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::algo::Algorithm;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// client.algo("codeb34v3r/FindMinMax/0.1")
    ///     .timeout(3)
    ///     .pipe(vec![2,3,4]);
    /// ```
    pub fn timeout(&mut self, timeout: u32) -> &mut Algorithm {
        self.options.timeout(timeout);
        self
    }

    /// Builder method to include stdout in the response metadata
    ///
    /// This has no affect unless authenticated as the owner of the algorithm
    pub fn enable_stdout(&mut self) -> &mut Algorithm {
        self.options.enable_stdout();
        self
    }
}

impl <'a> AlgoInput<'a> {
    /// If the `AlgoInput` is text (or a valid JSON string), returns the associated text
    pub fn as_string(&'a self) -> Option<&'a str> {
        match self {
            &AlgoInput::Text(ref text) => Some(&*text),
            &AlgoInput::Binary(_) => None,
            &AlgoInput::Json(Cow::Borrowed(&Value::String(ref text))) => Some(&*text),
            &AlgoInput::Json(Cow::Owned(Value::String(ref text))) => Some(&*text),
            _ => None,
        }
    }

    /// If the `AlgoInput` is Json (or text that can be JSON encoded), returns the associated JSON string
    ///
    /// For `AlgoInput::Json`, this returns the borrowed `Json`.
    ///   For the `AlgoInput::Text` variant, the text is wrapped into an owned `Json::String`.
    pub fn as_json(&'a self) -> Option<Cow<'a, Value>> {
        match self {
            &AlgoInput::Text(ref text) => Some(Cow::Owned(Value::String(text.clone().into_owned()))),
            &AlgoInput::Json(ref json) => Some(Cow::Borrowed(json)),
            &AlgoInput::Binary(_) => None,
        }
    }

    /// If the `AlgoInput` is binary, returns the associated byte slice
    pub fn as_bytes(&'a self) -> Option<&'a [u8]> {
        match self {
            &AlgoInput::Text(_) => None,
            &AlgoInput::Json(_) => None,
            &AlgoInput::Binary(ref bytes) => Some(&*bytes),
        }
    }

    /// If the `AlgoInput` is valid JSON, decode it to a particular type
    pub fn decode<D: Deserialize>(&self) -> Result<D, Error> {
        let res_json = try!(self.as_json()
            .ok_or(Error::ContentTypeError("Input is not JSON".into())));
        serde_json::from_value::<D>(res_json.into_owned()).map_err(|err| err.into())
    }
}

impl AlgoResponse {
    /// If the result is text (or a valid JSON string), returns the associated string
    pub fn as_string(self) -> Option<String> {
        match self.result {
            AlgoOutput::Text(text) => Some(text),
            AlgoOutput::Json(Value::String(text)) => Some(text),
            _ => None,
        }
    }

    /// If the result is JSON (or text that can be JSON encoded), returns the associated JSON `Value`
    pub fn as_json(self) -> Option<Value> {
        match self.result {
            AlgoOutput::Json(json) => Some(json),
            AlgoOutput::Text(text) => Some(Value::String(text)),
            _ => None,
        }
    }

    /// If the result is Binary, returns the associated byte slice
    pub fn as_bytes(self) -> Option<Vec<u8>> {
        match self.result {
            AlgoOutput::Binary(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// If the result is valid JSON, decode it to a particular type
    pub fn decode<D: Deserialize>(self) -> Result<D, Error> {
        let ct = self.metadata.content_type.clone();
        let res_json = try!(self.as_json()
            .ok_or(Error::ContentTypeError(ct)));
        serde_json::from_value::<D>(res_json).map_err(|err| err.into())
    }
}

impl AlgoOptions {
    pub fn new() -> AlgoOptions {
        AlgoOptions{ opts: HashMap::new() }
    }

    /// Configure timeout in seconds
    pub fn timeout(&mut self, timeout: u32) {
        self.opts.insert("timeout".into(), timeout.to_string());
    }

    /// Sets the option to enable stdout retrieval
    ///
    /// This has no affect unless authenticated as the owner of the algorithm
    pub fn enable_stdout(&mut self) {
        self.opts.insert("stdout".into(), true.to_string());
    }
}

impl Deref for AlgoOptions {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &HashMap<String, String> { &self.opts }
}

impl DerefMut for AlgoOptions {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.opts }
}

impl FromStr for AlgoResponse {
    type Err = Error;
    fn from_str(json_str: &str) -> Result<Self, Self::Err> {
        // Early return if the response decodes into ApiErrorResponse
        if let Ok(err_res) = serde_json::from_str::<ApiErrorResponse>(json_str) {
            return Err(err_res.error.into())
        }

        // Parse into Json object
        let data = try!(Value::from_str(json_str));

        // Construct the AlgoMetadata object
        let metadata = match data.search("metadata") {
            Some(meta_json) => try!(serde_json::from_str::<AlgoMetadata>(&meta_json.to_string())),
            None => {
                return Err(JsonError::Syntax(
                    serde_json::ErrorCode::MissingField("metadata"), 0, 0
                ).into());
            }
        };

        // Construct the AlgoOutput object
        let result = match (&*metadata.content_type, data.search("result")) {
            ("void", _) => AlgoOutput::Json(Value::Null),
            ("json", Some(json)) => AlgoOutput::Json(json.clone()), // TODO: Consider Cow<'a Json>
            ("text", Some(json)) => match json.as_str() {
                Some(text) => AlgoOutput::Text(text.into()),
                None => return Err(Error::ContentTypeError("invalid text".into())),
            },
            ("binary", Some(json)) => match json.as_str() {
                Some(text) => AlgoOutput::Binary(try!(base64::decode(text))),
                None => return Err(Error::ContentTypeError("invalid text".into())),
            },
            (_, None) => return Err(JsonError::Syntax(
                serde_json::ErrorCode::MissingField("result"), 0, 0
            ).into()),
            (content_type, _) => return Err(Error::ContentTypeError(content_type.into())),
        };

        // Construct the AlgoResponse object
        Ok(AlgoResponse{ metadata: metadata, result: result})
    }
}

impl fmt::Display for AlgoResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.result {
            AlgoOutput::Text(ref s) => f.write_str(s),
            AlgoOutput::Json(ref s) => f.write_str(&s.to_string()),
            AlgoOutput::Binary(ref bytes) => f.write_str(&String::from_utf8_lossy(bytes)),
        }
    }
}

impl Read for AlgoResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut out = buf; // why do I need this binding?
        match self.result {
            AlgoOutput::Text(ref s) => out.write(s.as_bytes()),
            AlgoOutput::Json(ref s) => out.write(s.to_string().as_bytes()),
            AlgoOutput::Binary(ref bytes) => out.write(bytes),
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

// AlgoInput Conversions
impl <'a> From<()> for AlgoInput<'a> {
    fn from(_unit: ()) -> Self {
        AlgoInput::Json(Cow::Owned(Value::Null))
    }
}

impl <'a> From<&'a str> for AlgoInput<'a> {
    fn from(text: &'a str) -> Self {
        AlgoInput::Text(Cow::Borrowed(text))
    }
}

impl <'a> From<&'a [u8]> for AlgoInput<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        AlgoInput::Binary(Cow::Borrowed(bytes))
    }
}

impl <'a> From<String> for AlgoInput<'a> {
    fn from(text: String) -> Self {
        AlgoInput::Text(Cow::Owned(text))
    }
}

impl <'a> From<Vec<u8>> for AlgoInput<'a> {
    fn from(bytes: Vec<u8>) -> Self {
        AlgoInput::Binary(Cow::Owned(bytes))
    }
}

impl <'a> From<Value> for AlgoInput<'a> {
    fn from(json: Value) -> Self {
        AlgoInput::Json(Cow::Owned(json))
    }
}

impl <'a, S: Serialize> From<&'a S> for AlgoInput<'a> {
    fn from(object: &'a S) -> Self {
        AlgoInput::Json(Cow::Owned(object.to_json()))
    }
}

// AlgoOutput conversions - could probably combine with fancier implementations
impl From<()> for AlgoOutput {
    fn from(_unit: ()) -> Self {
        AlgoOutput::Json(Value::Null)
    }
}

impl <'a> From<&'a str> for AlgoOutput {
    fn from(text: &'a str) -> Self {
        AlgoOutput::Text(text.into())
    }
}

impl From<String> for AlgoOutput {
    fn from(text: String) -> Self {
        AlgoOutput::Text(text)
    }
}

impl <'a> From<&'a [u8]> for AlgoOutput {
    fn from(bytes: &'a [u8]) -> Self {
        AlgoOutput::Binary(bytes.into())
    }
}

impl From<Vec<u8>> for AlgoOutput {
    fn from(bytes: Vec<u8>) -> Self {
        AlgoOutput::Binary(bytes)
    }
}

impl From<Value> for AlgoOutput {
    fn from(json: Value) -> Self {
        AlgoOutput::Json(json)
    }
}

impl <'a, S: Serialize> From<&'a S> for AlgoOutput {
    fn from(object: &'a S) -> Self {
        AlgoOutput::Json(object.to_json())
    }
}

// Add when overlapping specialization is possible
// impl <S: Serialize> From<S> for AlgoOutput {
//     fn from(object: S) -> Self {
//         AlgoOutput::Json(object.to_json())
//     }
// }

// The conversion that makes it easy to pipe output to another algorithm's input
impl <'a> From<AlgoOutput> for AlgoInput<'a> {
    fn from(output: AlgoOutput) -> Self {
        match output {
            AlgoOutput::Text(text) => AlgoInput::Text(Cow::Owned(text)),
            AlgoOutput::Json(json) => AlgoInput::Json(Cow::Owned(json)),
            AlgoOutput::Binary(bytes) => AlgoInput::Binary(Cow::Owned(bytes)),
        }
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
