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
//! let result: Vec<f64> = moving_avg.pipe(&input, None).unwrap().result().unwrap();
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
use std::fmt;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

static ALGORITHM_BASE_PATH: &'static str = "v1/algo";

pub enum AlgoInput<'a> {
    Text(&'a str),
    Binary(&'a [u8]),
    Json(Cow<'a, str>)
}


/// Algorithmia algorithm
pub struct Algorithm {
    pub path: String,
    client: HttpClient,
}


pub struct AlgoOptions {
    opts: HashMap<String, String>
}

pub struct AlgoRef {
    pub path: String
}

#[derive(RustcDecodable, Debug)]
pub struct AlgoMetadata {
    pub duration: f32,
    pub stdout: Option<String>,
    pub alerts: Option<Vec<String>>,
    pub content_type: String,
}

pub struct AlgoResponse {
    pub metadata: AlgoMetadata,
    result: AlgoOutput,
}

pub enum AlgoOutput {
    Void,
    Text(String),
    Json(String),
    Binary(Vec<u8>),
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
    ///     Ok(response) => println!("{}", response.result_json().unwrap()),
    ///     Err(err) => println!("ERROR: {}", err),
    /// };
    /// ```
    pub fn pipe<'a, I>(&'a self, input_data: I, options: Option<&'a AlgoOptions>)
                       -> Result<AlgoResponse, Error> where
        I: Into<AlgoInput<'a>>
    {
        let mut res = try!(match input_data.into() {
            AlgoInput::Text(text) => self.pipe_as(text, Mime(TopLevel::Text, SubLevel::Plain, vec![]), options),
            AlgoInput::Json(json) => self.pipe_as(&*json, Mime(TopLevel::Application, SubLevel::Json, vec![]), options),
            AlgoInput::Binary(bytes) => self.pipe_as(bytes, Mime(TopLevel::Application, SubLevel::Ext("octet-stream".into()), vec![]), options),
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
    ///    Ok(response) => response.result_json().unwrap().to_owned(),
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



impl AlgoOptions {
    pub fn new() -> AlgoOptions {
        AlgoOptions { opts: HashMap::new() }
    }

    pub fn timeout(&mut self, timeout: u32) {
        self.opts.insert("timeout".into(), timeout.to_string());
    }

    pub fn stdout(&mut self, stdout: bool) {
        self.opts.insert("stdout".into(), stdout.to_string());
    }
}



impl AlgoResponse {
    pub fn result_json(&self) -> Result<&str, Error> {
        match self.result {
            AlgoOutput::Json(ref json) => Ok(&json),
            _ => Err(Error::ContentTypeError(self.metadata.content_type.clone())),
        }
    }

    pub fn result_bytes(&self) -> Result<&[u8], Error> {
        match self.result {
            AlgoOutput::Binary(ref bytes) => Ok(&bytes),
            _ => Err(Error::ContentTypeError(self.metadata.content_type.clone())),
        }
    }

    pub fn result_str(&self) -> Result<&str, Error> {
        match self.result {
            AlgoOutput::Text(ref text) => Ok(&text),
            _ => Err(Error::ContentTypeError(self.metadata.content_type.clone())),
        }
    }

    pub fn result<D: Decodable>(&self) -> Result<D, Error> {
        let res_json = try!(self.result_json());
        json::decode::<D>(&res_json).map_err(|err| err.into())
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

        // Construct the AlgoOutput object
        let result = match (&*metadata.content_type, data.search("result")) {
            ("void", _) => AlgoOutput::Void,
            ("json", Some(json)) => AlgoOutput::Json(json.to_string()),
            ("text", Some(json)) => match json.as_string() {
                Some(text) => AlgoOutput::Text(text.into()),
                None => return Err(Error::ContentTypeError("invalid text".into())),
            },
            ("binary", Some(json)) => match json.as_string() {
                Some(text) => AlgoOutput::Binary(try!(text.from_base64())),
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
            AlgoOutput::Void => Err(fmt::Error),
            AlgoOutput::Text(ref s) | AlgoOutput::Json(ref s) => f.write_str(s),
            AlgoOutput::Binary(ref bytes) => f.write_str(&String::from_utf8_lossy(bytes)),
        }
    }
}

impl Read for AlgoResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut out = buf; // why do I need this binding?
        match self.result {
            AlgoOutput::Void => Err(io::Error::new(io::ErrorKind::Other, "cannot read void content type")),
            AlgoOutput::Text(ref s) | AlgoOutput::Json(ref s) => out.write(s.as_bytes()),
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

impl <'a> From<&'a str> for AlgoInput<'a> {
    fn from(text: &'a str) -> Self {
        AlgoInput::Text(text)
    }
}

impl <'a> From<&'a [u8]> for AlgoInput<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        AlgoInput::Binary(bytes)
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


impl Deref for AlgoOptions {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &HashMap<String, String> { &self.opts }
}

impl DerefMut for AlgoOptions {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.opts }
}

impl <'a, E: Encodable> From<&'a E> for AlgoInput<'a> {
    fn from(encodable: &'a E) -> Self {
        // TODO: remove unwrap - either find a way to Box the encodable object and let pipe() encode it
        //       or store a result and let pipe() do error handling
        AlgoInput::Json(json::encode(encodable).unwrap().into())
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
        assert_eq!(expected_result, &*decoded.result::<Vec<i32>>().unwrap());
    }
}
