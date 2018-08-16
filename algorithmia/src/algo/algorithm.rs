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
use Body;

use serde_json::{self, Value};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde::de::Error as SerdeError;

use base64;
use reqwest::header::ContentType;
use reqwest::Url;
use mime::{self, Mime};
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
pub enum AlgoInput {
    /// Data that will be sent with `Content-Type: text/plain`
    Text(String),
    /// Data that will be sent with `Content-Type: application/octet-stream`
    Binary(Vec<u8>),
    /// Data that will be sent with `Content-Type: application/json`
    Json(Value),
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
#[derive(Debug, Deserialize)]
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
    #[serde(skip_deserializing)]
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
        base_url
            .join(&path)
            .chain_err(|| ErrorKind::InvalidAlgoUri(path))
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
    pub fn pipe<I>(&self, input_data: I) -> Result<AlgoResponse>
    where
        I: Into<AlgoInput>,
    {
        let mut res = match input_data.into() {
            AlgoInput::Text(text) => self.pipe_as(text, mime::TEXT_PLAIN)?,
            AlgoInput::Json(json) => {
                let encoded = serde_json::to_vec(&json)
                    .chain_err(|| ErrorKind::EncodeJson("algorithm input"))?;
                self.pipe_as(encoded, mime::APPLICATION_JSON)?
            }
            AlgoInput::Binary(bytes) => self.pipe_as(bytes, mime::APPLICATION_OCTET_STREAM)?,
        };

        let mut res_json = String::new();
        res.read_to_string(&mut res_json)
            .chain_err(|| "failed to read algorithm response")?;
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
        let mut res = self.pipe_as(json_input.to_owned(), mime::APPLICATION_JSON)?;

        let mut res_json = String::new();
        res.read_to_string(&mut res_json)
            .chain_err(|| "failed to read algorithm response")?;
        res_json.parse()
    }


    #[doc(hidden)]
    pub fn pipe_as<B>(&self, input_data: B, content_type: Mime) -> Result<Response>
    where
        B: Into<Body>,
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
        self.client
            .post(url)
            .header(ContentType(content_type))
            .body(input_data)
            .send().chain_err(|| {
                ErrorKind::Http(format!("calling algorithm '{}'", self.algo_uri))
            })
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
    /// Returns the algorithm's URI path
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl AlgoInput {
    /// If the `AlgoInput` is text (or a valid JSON string), returns the associated text
    #[allow(match_same_arms)]
    pub fn as_string(&self) -> Option<&str> {
        match *self {
            AlgoInput::Text(ref text) => Some(&*text),
            AlgoInput::Json(ref json) => json.as_str(),
            _ => None,
        }
    }

    /// If the `AlgoInput` is Json (or JSON encodable text), returns the associated JSON string
    ///
    /// For `AlgoInput::Json`, this returns the borrowed `Json`.
    ///   For the `AlgoInput::Text` variant, the text is wrapped into an owned `Value::String`.
    pub fn as_json<'a>(&'a self) -> Option<Cow<'a, Value>> {
        match *self {
            AlgoInput::Text(ref text) => Some(Cow::Owned(Value::String(text.to_owned()))),
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
    pub fn decode<D: DeserializeOwned>(&self) -> Result<D> {
        let res_json = self.as_json()
            .ok_or(ErrorKind::MismatchedContentType("json"))?;
        serde_json::from_value(res_json.into_owned())
            .chain_err(|| "failed to decode input to specified type")
    }
}

impl AlgoResponse {
    /// Return algorithm output as a string (if text or a valid JSON string)
    #[allow(match_same_arms)]
    pub fn into_string(self) -> Option<String> {
        match self.result {
            AlgoOutput::Text(text) => Some(text),
            AlgoOutput::Json(Value::String(text)) => Some(text),
            _ => None,
        }
    }

    /// Read algorithm output as JSON `Value` (if JSON of text)
    pub fn into_json(self) -> Option<Value> {
        match self.result {
            AlgoOutput::Json(json) => Some(json),
            AlgoOutput::Text(text) => Some(Value::String(text)),
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
    pub fn decode<D>(self) -> Result<D>
    where
        for<'de> D: Deserialize<'de>,
    {
        let ct = self.metadata.content_type.clone();
        let res_json = self.into_json()
            .ok_or_else(|| ErrorKind::UnexpectedContentType("json", ct))?;
        serde_json::from_value(res_json).chain_err(|| ErrorKind::DecodeJson("algorithm response"))
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
        if let Ok(err_res) = serde_json::from_str::<ApiErrorResponse>(json_str) {
            return Err(ErrorKind::Api(err_res.error).into());
        }

        // Parse into Json object
        let mut data = Value::from_str(json_str)
            .chain_err(|| ErrorKind::DecodeJson("algorithm response"))?;
        let metadata_value = data.as_object_mut()
            .and_then(|ref mut o| o.remove("metadata"))
            .ok_or_else(|| serde_json::Error::missing_field("metadata"))
            .chain_err(|| ErrorKind::DecodeJson("algorithm response"))?;
        let result_value = data.as_object_mut()
            .and_then(|ref mut o| o.remove("result"))
            .ok_or_else(|| serde_json::Error::missing_field("result"))
            .chain_err(|| ErrorKind::DecodeJson("algorithm response"))?;

        // Construct the AlgoOutput object
        let metadata = serde_json::from_value::<AlgoMetadata>(metadata_value)
            .chain_err(|| ErrorKind::DecodeJson("algorithm response metadata"))?;
        let result = match (&*metadata.content_type, result_value) {
            ("void", _) => AlgoOutput::Json(Value::Null),
            ("json", value) => AlgoOutput::Json(value),
            ("text", value) => {
                match value.as_str() {
                    Some(text) => AlgoOutput::Text(text.into()),
                    None => return Err(ErrorKind::MismatchedContentType("text").into()),
                }
            }
            ("binary", value) => {
                match value.as_str() {
                    Some(text) => {
                        let binary =
                            base64::decode(text)
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
impl From<()> for AlgoInput {
    fn from(_unit: ()) -> Self {
        AlgoInput::Json(Value::Null)
    }
}

impl<'a> From<&'a str> for AlgoInput {
    fn from(text: &'a str) -> Self {
        AlgoInput::Text(text.to_owned())
    }
}

impl<'a> From<&'a [u8]> for AlgoInput {
    fn from(bytes: &'a [u8]) -> Self {
        AlgoInput::Binary(bytes.to_owned())
    }
}

impl From<String> for AlgoInput {
    fn from(text: String) -> Self {
        AlgoInput::Text(text.to_owned())
    }
}

impl From<Vec<u8>> for AlgoInput {
    fn from(bytes: Vec<u8>) -> Self {
        AlgoInput::Binary(bytes)
    }
}

impl From<Value> for AlgoInput {
    fn from(json: Value) -> Self {
        AlgoInput::Json(json)
    }
}

impl<'a, S: Serialize> From<&'a S> for AlgoInput {
    fn from(object: &'a S) -> Self {
        AlgoInput::Json(
            serde_json::to_value(object).expect("Failed to serialize"),
        )
    }
}

// AlgoOutput conversions
impl From<()> for AlgoOutput {
    fn from(_unit: ()) -> Self {
        AlgoOutput::Json(Value::Null)
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

impl From<Value> for AlgoOutput {
    fn from(json: Value) -> Self {
        AlgoOutput::Json(json)
    }
}

impl<'a, S: Serialize> From<&'a S> for AlgoOutput {
    fn from(object: &'a S) -> Self {
        AlgoOutput::Json(serde_json::to_value(object).expect("Failed to serialize"))
    }
}

impl<S: Serialize> From<Box<S>> for AlgoOutput {
    fn from(object: Box<S>) -> Self {
        AlgoOutput::Json(serde_json::to_value(object).expect("Failed to serialize"))
    }
}

// Waiting for specialization to stabilize
#[cfg(feature = "nightly")]
impl<S: Serialize> From<S> for AlgoOutput {
    default fn from(object: S) -> Self {
        AlgoOutput::Json(serde_json::to_value(object).expect("Failed to serialize"))
    }
}

// The conversion that makes it easy to pipe output to another algorithm's input
impl From<AlgoOutput> for AlgoInput {
    fn from(output: AlgoOutput) -> Self {
        match output {
            AlgoOutput::Text(text) => AlgoInput::Text(text),
            AlgoOutput::Json(json) => AlgoInput::Json(json),
            AlgoOutput::Binary(bytes) => AlgoInput::Binary(bytes),
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
        assert_eq!(
            algorithm.to_url().unwrap().path(),
            "/v1/algo/anowell/Pinky/0.1.0"
        );
    }

    #[test]
    fn test_algo_with_prefix_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo("algo://anowell/Pinky/0.1");
        assert_eq!(
            algorithm.to_url().unwrap().path(),
            "/v1/algo/anowell/Pinky/0.1"
        );
    }

    #[test]
    fn test_algo_typesafe_to_url() {
        let mock_client = mock_client();
        let pinky = AlgoUri::with_version("anowell/Pinky", "abcdef123456");
        let algorithm = mock_client.algo(pinky);
        assert_eq!(
            algorithm.to_url().unwrap().path(),
            "/v1/algo/anowell/Pinky/abcdef123456"
        );
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
