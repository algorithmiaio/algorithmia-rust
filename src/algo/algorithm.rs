//! Algorithm module for executing Algorithmia algorithms
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Algorithmia;
//!
//! // Initialize with an API key
//! let client = Algorithmia::client("111112222233333444445555566");
//! let moving_avg = client.algo("timeseries/SimpleMovingAverage/0.1");
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = (vec![0,1,2,3,15,4,5,6,7], 3);
//! let result: Vec<f64> = moving_avg.pipe(&input).unwrap().decode().unwrap();
//! println!("Completed with result: {:?}", result);
//! ```

use client::HttpClient;
use error::{Error, ErrorKind, Result, ResultExt, ApiErrorResponse};
use super::version::Version;
use {json, Body};

#[cfg(feature="with-serde")]
use serde_json::{self, Value};
#[cfg(feature="with-serde")]
use serde_json::value::ToJson;
#[cfg(feature="with-serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature="with-rustc-serialize")]
use rustc_serialize::{self, Decodable, Encodable};
#[cfg(feature="with-rustc-serialize")]
use rustc_serialize::json::Json;

#[cfg(feature="with-serde")]
macro_rules! JsonValue {
    ($i:ident) => { serde_json::Value::$i };
    ($i:ident, $e:expr) => { serde_json::Value::$i($e) };
}

#[cfg(feature="with-rustc-serialize")]
#[deprecated(since="2.1.0", note="rustc-serialize has been deprecated")]
macro_rules! JsonValue {
    ($i:ident) => { rustc_serialize::json::Json::$i };
    ($i:ident, $e:expr) => { rustc_serialize::json::Json::$i($e) };
}

/// Feature-specific alias to a JSON enum type
///
/// This type alias varies by compilation feature to use the
///   JSON enum type of the underlying JSON crate.
#[cfg(feature="with-serde")]
pub type JsonValue = Value;

/// Feature-specific alias to a JSON enum type
///
/// This type alias varies by compilation feature to use the
///   JSON enum type of the underlying JSON crate.
#[cfg(feature="with-rustc-serialize")]
#[deprecated(since="2.1.0", note="rustc-serialize has been deprecated")]
pub type JsonValue = Json;

use base64;
use reqwest::header::ContentType;
use reqwest::Url;
use mime::Mime;
#[doc(hidden)]
pub use reqwest::Response;

use std::borrow::Cow;
use std::io::{self, Read, Write};
use std::str::FromStr;
use std::fmt;
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
    Json(Cow<'a, JsonValue>),
}

/// Types that can store the output of an algorithm
pub enum AlgoOutput {
    /// Representation of result when `metadata.content_type` is 'text'
    Text(String),
    /// Representation of result when `metadata.content_type` is 'json'
    Json(JsonValue),
    /// Representation of result when `metadata.content_type` is 'binary'
    Binary(Vec<u8>),
}

/// Algorithmia algorithm - intialized from the `Algorithmia` builder
pub struct Algorithm {
    algo_uri: AlgoUri,
    options: AlgoOptions,
    client: HttpClient,
}

/// Options used to alter the algorithm call, e.g. configuring the timeout
pub struct AlgoOptions {
    opts: HashMap<String, String>,
}

/// URI of an Algorithmia algorithm
#[derive(Clone)]
pub struct AlgoUri {
    path: String,
}

/// Metadata returned from the API
#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable))]
#[derive(Debug)]
pub struct AlgoMetadata {
    /// Algorithm execution duration
    pub duration: f32,
    /// Stdout from the algorithm (must enable stdout on request and be the algorithm author)
    pub stdout: Option<String>,
    /// API alerts (e.g. low balance warning)
    pub alerts: Option<Vec<String>>,
    /// Describes how the ouput's `result` field should be parsed (`text`, `json`, or `binary`)
    pub content_type: String,
    // Placeholder for API stability if additional fields are added later
    #[cfg_attr(feature="with-serde", serde(skip_deserializing))]
    _dummy: (),
}

/// Successful API response that wraps the `AlgoOutput` and its Metadata
pub struct AlgoResponse {
    /// Any metadata associated with the API response
    pub metadata: AlgoMetadata,
    /// The algorithm output decoded into an `AlgoOutput` enum
    pub result: AlgoOutput,
    // Placeholder for API stability if additional fields are added later
    _dummy: (),
}


impl Algorithm {
    #[doc(hidden)]
    pub fn new(client: HttpClient, algo_uri: AlgoUri) -> Algorithm {
        Algorithm {
            client: client,
            algo_uri: algo_uri,
            options: AlgoOptions::default(),
        }
    }

    /// Get the API Endpoint URL for this Algtried orithm
    pub fn to_url(&self) -> Result<Url> {
        let base_url = self.client
            .base_url
            .as_ref()
            .map_err(|err| *err)
            .chain_err(|| ErrorKind::InvalidBaseUrl)?;
        let path = format!("{}/{}", ALGORITHM_BASE_PATH, self.algo_uri.path);
        base_url.join(&path).chain_err(|| ErrorKind::InvalidAlgoUri(path))
    }

    /// Get the Algorithmia algo URI for this Algorithm
    pub fn to_algo_uri(&self) -> &AlgoUri {
        &self.algo_uri
    }

    /// Execute an algorithm with the specified `input_data`.
    ///
    /// `input_data` can be any type which converts into `AlgoInput`,
    ///   including strings, byte slices, and any serializable type.
    ///   To create serializable objects for complex input, annotate your type
    ///   with `#[derive(Serialize)]` (see [serde.rs](http://serde.rs) for details).
    ///   If you want to send a raw, unparsed JSON string, use the `pipe_json` method instead.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let moving_avg = client.algo("timeseries/SimpleMovingAverage/0.1");
    /// let input = (vec![0,1,2,3,15,4,5,6,7], 3);
    /// match moving_avg.pipe(&input) {
    ///     Ok(response) => println!("{}", response.into_json().unwrap()),
    ///     Err(err) => println!("ERROR: {}", err),
    /// };
    /// ```
    pub fn pipe<'a, I>(&'a self, input_data: I) -> Result<AlgoResponse>
        where I: Into<AlgoInput<'a>>
    {
        let mut res = match input_data.into() {
            AlgoInput::Text(text) => self.pipe_as(&*text, mime!(Text / Plain))?,
            AlgoInput::Json(json) => {
                let encoded =
                    json::encode(&json).chain_err(|| ErrorKind::EncodeJson("algorithm input"))?;
                self.pipe_as(&*encoded, mime!(Application / Json))?
            }
            AlgoInput::Binary(bytes) => self.pipe_as(&*bytes, mime!(Application / OctetStream))?,
        };

        let mut res_json = String::new();
        res.read_to_string(&mut res_json).chain_err(|| "failed to read algorithm response")?;
        res_json.parse()
    }

    /// Execute an algorithm with a raw JSON string as input.
    ///
    /// While the `pipe` method is more flexible in accepting different types
    ///   of input, and inferring the content type when making an API call,
    ///   `pipe_json` explicitly sends the provided string with
    ///   `Content-Type: application/json` making no attempt to verify that
    ///   the input is valid JSON. By contrast, calling `pipe` with a string
    ///   would send it with `Content-Type: text/plain`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let minmax  = client.algo("codeb34v3r/FindMinMax/0.1");
    ///
    /// let output = match minmax.pipe_json("[2,3,4]") {
    ///    Ok(response) => response.into_json().unwrap(),
    ///    Err(err) => panic!("{}", err),
    /// };
    pub fn pipe_json(&self, json_input: &str) -> Result<AlgoResponse> {
        let mut res = self.pipe_as(json_input, mime!(Application / Json))?;

        let mut res_json = String::new();
        res.read_to_string(&mut res_json).chain_err(|| "failed to read algorithm response")?;
        res_json.parse()
    }


    #[doc(hidden)]
    pub fn pipe_as<B>(&self, input_data: B, content_type: Mime) -> Result<Response>
        where B: Into<Body>
    {

        // Append options to URL as query parameters
        let mut url = self.to_url()?;
        if !self.options.is_empty() {
            let mut query_params = url.query_pairs_mut();
            for (k, v) in self.options.iter() {
                query_params.append_pair(&*k, &*v);
            }
        }

        // We just need the path and query string
        let req = self.client
            .post(url)
            .header(ContentType(content_type))
            .body(input_data);

        req.send().chain_err(|| ErrorKind::Http(format!("calling algorithm '{}'", self.algo_uri)))
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
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// client.algo("codeb34v3r/FindMinMax/0.1")
    ///     .timeout(3)
    ///     .pipe(vec![2,3,4])
    ///     .unwrap();
    /// ```
    pub fn timeout(&mut self, timeout: u32) -> &mut Algorithm {
        self.options.timeout(timeout);
        self
    }

    /// Builder method to enabled or disable stdout in the response metadata
    ///
    /// This has no affect unless authenticated as the owner of the algorithm
    pub fn stdout(&mut self, stdout: bool) -> &mut Algorithm {
        self.options.stdout(stdout);
        self
    }
}

impl AlgoUri {
    /// Initialize `AlgoUri` with a typed `Version`
    ///
    /// ```
    /// # use algorithmia::algo::{AlgoUri, Version};
    /// let uri = AlgoUri::with_version("demo/Hello", Version::Minor(0, 1));
    /// assert_eq!(uri.path(), "demo/Hello/0.1");
    /// ```
    pub fn with_version<V: Into<Version>>(user_algo: &str, version: V) -> AlgoUri {
        let path = match version.into() {
            Version::Latest => user_algo.to_owned(),
            ref ver => format!("{}/{}", user_algo, ver),
        };

        AlgoUri { path: path }
    }


    /// Returns the algorithm's URI path
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl<'a> AlgoInput<'a> {
    /// If the `AlgoInput` is text (or a valid JSON string), returns the associated text
    #[allow(match_same_arms)]
    pub fn as_string(&self) -> Option<&str> {
        match *self {
            AlgoInput::Text(ref text) => Some(&*text),
            AlgoInput::Json(Cow::Borrowed(json)) => json::value_as_str(json),
            AlgoInput::Json(Cow::Owned(ref json)) => json::value_as_str(json),
            _ => None,
        }
    }

    /// If the `AlgoInput` is Json (or JSON encodable text), returns the associated JSON string
    ///
    /// For `AlgoInput::Json`, this returns the borrowed `Json`.
    ///   For the `AlgoInput::Text` variant, the text is wrapped into an owned `Json::String`.
    pub fn as_json(&'a self) -> Option<Cow<'a, JsonValue>> {
        match *self {
            AlgoInput::Text(ref text) => {
                Some(Cow::Owned(JsonValue!(String, text.clone().into_owned())))
            }
            AlgoInput::Json(ref json) => Some(Cow::Borrowed(json)),
            AlgoInput::Binary(_) => None,
        }
    }

    /// If the `AlgoInput` is binary, returns the associated byte slice
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match *self {
            AlgoInput::Text(_) |
            AlgoInput::Json(_) => None,
            AlgoInput::Binary(ref bytes) => Some(&*bytes),
        }
    }


    /// If the `AlgoInput` is valid JSON, decode it to a particular type
    #[cfg(feature="with-serde")]
    pub fn decode<D: Deserialize>(&self) -> Result<D> {
        let res_json = self.as_json().ok_or(ErrorKind::MismatchedContentType("json"))?;
        json::decode_value::<D>(res_json.into_owned())
            .chain_err(|| "failed to decode input to specified type")
    }

    /// If the `AlgoInput` is valid JSON, decode it to a particular type
    #[cfg(feature="with-rustc-serialize")]
    #[deprecated(since="2.1.0", note="rustc-serialize has been deprecated")]
    pub fn decode<D: Decodable>(&self) -> Result<D> {
        let res_json = self.as_json().ok_or(ErrorKind::MismatchedContentType("json"))?;
        json::decode_value::<D>(res_json.into_owned())
            .chain_err(|| "failed to decode input to specified type")
    }
}

impl AlgoResponse {
    /// Return algorithm output as a string (if text or a valid JSON string)
    #[allow(match_same_arms)]
    pub fn into_string(self) -> Option<String> {
        match self.result {
            AlgoOutput::Text(text) => Some(text),
            #[cfg(feature="with-serde")] AlgoOutput::Json(Value::String(text)) => Some(text),
            #[cfg(feature="with-rustc-serialize")] AlgoOutput::Json(Json::String(text)) => Some(text),
            _ => None,
        }
    }

    /// Read algorithm output as `JsonValue` (if JSON of text)
    pub fn into_json(self) -> Option<JsonValue> {
        match self.result {
            AlgoOutput::Json(json) => Some(json),
            AlgoOutput::Text(text) => Some(JsonValue!(String, text)),
            _ => None,
        }
    }

    /// If the algorithm output is Binary, returns the associated byte slice
    pub fn into_bytes(self) -> Option<Vec<u8>> {
        match self.result {
            AlgoOutput::Binary(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// If the algorithm output is JSON, decode it into a particular type
    #[cfg(feature="serde")]
    pub fn decode<D: Deserialize>(self) -> Result<D> {
        let ct = self.metadata.content_type.clone();
        let res_json = self.into_json()
            .ok_or_else(|| ErrorKind::UnexpectedContentType("json", ct))?;
        json::decode_value::<D>(res_json).chain_err(|| ErrorKind::DecodeJson("algorithm response"))
    }

    /// If the algorithm output is JSON, decode it into a particular type
    #[cfg(feature="with-rustc-serialize")]
    #[deprecated(since="2.1.0", note="rustc-serialize has been deprecated")]
    pub fn decode<D: Decodable>(self) -> Result<D> {
        let ct = self.metadata.content_type.clone();
        let res_json = self.into_json()
            .ok_or_else(|| ErrorKind::UnexpectedContentType("json", ct))?;
        json::decode_value::<D>(res_json).chain_err(|| ErrorKind::DecodeJson("algorithm response"))
    }
}

impl AlgoOptions {
    /// Configure timeout in seconds
    pub fn timeout(&mut self, timeout: u32) {
        self.opts.insert("timeout".into(), timeout.to_string());
    }

    /// Enable or disable stdout retrieval
    ///
    /// This has no affect unless authenticated as the owner of the algorithm
    pub fn stdout(&mut self, stdout: bool) {
        self.opts.insert("stdout".into(), stdout.to_string());
    }
}

impl Default for AlgoOptions {
    fn default() -> AlgoOptions {
        AlgoOptions { opts: HashMap::new() }
    }
}

impl Deref for AlgoOptions {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &HashMap<String, String> {
        &self.opts
    }
}

impl DerefMut for AlgoOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.opts
    }
}

impl FromStr for AlgoResponse {
    type Err = Error;
    fn from_str(json_str: &str) -> ::std::result::Result<Self, Self::Err> {
        // Early return if the response decodes into ApiErrorResponse
        if let Ok(err_res) = json::decode_str::<ApiErrorResponse>(json_str) {
            return Err(ErrorKind::Api(err_res.error).into());
        }

        // Parse into Json object
        let mut data = json::value_from_str(json_str)
            .chain_err(|| ErrorKind::DecodeJson("algorithm response"))?;
        let metadata_value = json::take_field(&mut data, "metadata")
            .ok_or_else(|| json::missing_field_error("metadata"))
            .chain_err(|| ErrorKind::DecodeJson("algorithm response"))?;
        let result_value =
            json::take_field(&mut data, "result").ok_or_else(|| json::missing_field_error("result"))
                .chain_err(|| ErrorKind::DecodeJson("algorithm response"))?;

        // Construct the AlgoOutput object
        let metadata = json::decode_value::<AlgoMetadata>(metadata_value)
            .chain_err(|| ErrorKind::DecodeJson("algorithm response metadata"))?;
        let result = match (&*metadata.content_type, result_value) {
            ("void", _) => AlgoOutput::Json(JsonValue!(Null)),
            ("json", value) => AlgoOutput::Json(value),
            ("text", value) => {
                match json::value_as_str(&value) {
                    Some(text) => AlgoOutput::Text(text.into()),
                    None => return Err(ErrorKind::MismatchedContentType("text").into()),
                }
            }
            ("binary", value) => {
                match json::value_as_str(&value) {
                    Some(text) => {
                        let binary = base64::decode(text)
                            .chain_err(|| ErrorKind::DecodeBase64("algorithm response"))?;
                        AlgoOutput::Binary(binary)
                    }
                    None => return Err(ErrorKind::MismatchedContentType("binary").into()),
                }
            }
            (content_type, _) => {
                return Err(ErrorKind::InvalidContentType(content_type.into()).into())
            }
        };

        // Construct the AlgoResponse object
        Ok(AlgoResponse {
            metadata: metadata,
            result: result,
            _dummy: (),
        })
    }
}

impl fmt::Display for AlgoUri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.path)
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
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        match self.result {
            AlgoOutput::Text(ref s) => buf.write(s.as_bytes()),
            AlgoOutput::Json(ref s) => buf.write(s.to_string().as_bytes()),
            AlgoOutput::Binary(ref bytes) => buf.write(bytes),
        }
    }
}

impl<'a> From<&'a str> for AlgoUri {
    fn from(path: &'a str) -> Self {
        let path = match path {
            p if p.starts_with("algo://") => &p[7..],
            p if p.starts_with('/') => &p[1..],
            p => p,
        };
        AlgoUri { path: path.to_owned() }
    }
}

impl From<String> for AlgoUri {
    fn from(path: String) -> Self {
        let path = match path {
            ref p if p.starts_with("algo://") => p[7..].to_owned(),
            ref p if p.starts_with('/') => p[1..].to_owned(),
            p => p,
        };
        AlgoUri { path: path }
    }
}

// AlgoInput Conversions
impl<'a> From<()> for AlgoInput<'a> {
    fn from(_unit: ()) -> Self {
        AlgoInput::Json(Cow::Owned(JsonValue!(Null)))
    }
}

impl<'a> From<&'a str> for AlgoInput<'a> {
    fn from(text: &'a str) -> Self {
        AlgoInput::Text(Cow::Borrowed(text))
    }
}

impl<'a> From<&'a [u8]> for AlgoInput<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        AlgoInput::Binary(Cow::Borrowed(bytes))
    }
}

impl<'a> From<String> for AlgoInput<'a> {
    fn from(text: String) -> Self {
        AlgoInput::Text(Cow::Owned(text))
    }
}

impl<'a> From<Vec<u8>> for AlgoInput<'a> {
    fn from(bytes: Vec<u8>) -> Self {
        AlgoInput::Binary(Cow::Owned(bytes))
    }
}

impl<'a> From<JsonValue> for AlgoInput<'a> {
    fn from(json: JsonValue) -> Self {
        AlgoInput::Json(Cow::Owned(json))
    }
}

#[cfg(feature="with-serde")]
impl<'a, S: Serialize> From<&'a S> for AlgoInput<'a> {
    fn from(object: &'a S) -> Self {
        AlgoInput::Json(Cow::Owned(object.to_json().expect("Failed to serialize")))
    }
}

#[cfg(feature="with-rustc-serialize")]
#[deprecated(since="2.1.0", note="rustc-serialize has been deprecated")]
impl<'a, E: Encodable> From<&'a E> for AlgoInput<'a> {
    fn from(object: &'a E) -> Self {
        // Not great - but serde is the longer-term story anyway
        let encoded = json::encode(&object).unwrap();
        AlgoInput::Json(Cow::Owned(Json::from_str(&encoded).unwrap()))
    }
}

// AlgoOutput conversions
impl From<()> for AlgoOutput {
    fn from(_unit: ()) -> Self {
        AlgoOutput::Json(JsonValue!(Null))
    }
}

impl<'a> From<&'a str> for AlgoOutput {
    fn from(text: &'a str) -> Self {
        AlgoOutput::Text(text.into())
    }
}

impl From<String> for AlgoOutput {
    fn from(text: String) -> Self {
        AlgoOutput::Text(text)
    }
}

impl<'a> From<&'a [u8]> for AlgoOutput {
    fn from(bytes: &'a [u8]) -> Self {
        AlgoOutput::Binary(bytes.into())
    }
}

impl From<Vec<u8>> for AlgoOutput {
    fn from(bytes: Vec<u8>) -> Self {
        AlgoOutput::Binary(bytes)
    }
}

impl From<JsonValue> for AlgoOutput {
    fn from(json: JsonValue) -> Self {
        AlgoOutput::Json(json)
    }
}

#[cfg(feature="with-serde")]
impl<'a, S: Serialize> From<&'a S> for AlgoOutput {
    fn from(object: &'a S) -> Self {
        AlgoOutput::Json(object.to_json().expect("Failed to serialize"))
    }
}

#[cfg(feature="with-serde")]
impl<S: Serialize> From<Box<S>> for AlgoOutput {
    fn from(object: Box<S>) -> Self {
        AlgoOutput::Json(object.to_json().expect("Failed to serialize"))
    }
}

// Waiting for specialization to stabilize
#[cfg(all(feature="with-serde", feature="nightly"))]
impl<S: Serialize> From<S> for AlgoOutput {
    default fn from(object: S) -> Self {
        AlgoOutput::Json(object.to_json())
    }
}

#[cfg(feature="with-rustc-serialize")]
#[deprecated(since="2.1.0", note="rustc-serialize has been deprecated")]
impl<'a, E: Encodable> From<&'a E> for AlgoOutput {
    fn from(object: &'a E) -> Self {
        // Not great - but serde is the longer-term story anyway
        let encoded = json::encode(object).unwrap();
        AlgoOutput::Json(Json::from_str(&encoded).unwrap())
    }
}

#[cfg(feature="with-rustc-serialize")]
#[deprecated(since="2.1.0", note="rustc-serialize has been deprecated")]
impl<E: Encodable> From<Box<E>> for AlgoOutput {
    fn from(object: Box<E>) -> Self {
        // Not great - but serde is the longer-term story anyway
        let encoded = json::encode(&object).unwrap();
        AlgoOutput::Json(Json::from_str(&encoded).unwrap())
    }
}

// Waiting for specialization to stabilize
#[cfg(all(feature="with-rustc-serialize", feature="nightly"))]
#[deprecated(since="2.1.0", note="rustc-serialize has been deprecated")]
impl<E: Encodable> From<E> for AlgoOutput {
    default fn from(object: E) -> Self {
        // Not great - but serde is the longer-term story anyway
        let encoded = json::encode(&object).unwrap();
        AlgoOutput::Json(Json::from_str(&encoded).unwrap())
    }
}

// The conversion that makes it easy to pipe output to another algorithm's input
impl<'a> From<AlgoOutput> for AlgoInput<'a> {
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

    fn mock_client() -> Algorithmia {
        Algorithmia::client("")
    }

    #[test]
    fn test_algo_without_version_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo("/anowell/Pinky");
        assert_eq!(algorithm.to_url().unwrap().path(), "/v1/algo/anowell/Pinky");
    }

    #[test]
    fn test_algo_without_prefix_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo("anowell/Pinky/0.1.0");
        assert_eq!(algorithm.to_url().unwrap().path(),
                   "/v1/algo/anowell/Pinky/0.1.0");
    }

    #[test]
    fn test_algo_with_prefix_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo("algo://anowell/Pinky/0.1");
        assert_eq!(algorithm.to_url().unwrap().path(),
                   "/v1/algo/anowell/Pinky/0.1");
    }

    #[test]
    fn test_algo_typesafe_to_url() {
        let mock_client = mock_client();
        let pinky = AlgoUri::with_version("anowell/Pinky", "abcdef123456");
        let algorithm = mock_client.algo(pinky);
        assert_eq!(algorithm.to_url().unwrap().path(),
                   "/v1/algo/anowell/Pinky/abcdef123456");
    }


    #[test]
    fn test_json_decoding() {
        let json_output =
            r#"{"metadata":{"duration":0.46739511,"content_type":"json"},"result":[5,41]}"#;
        let expected_result = [5, 41];
        let decoded = json_output.parse::<AlgoResponse>().unwrap();
        assert_eq!(0.46739511f32, decoded.metadata.duration);
        assert_eq!(expected_result, &*decoded.decode::<Vec<i32>>().unwrap());
    }
}
