//! API client for calling Algorithmia algorithms
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Algorithmia;
//!
//! # fn main() -> Result<(), Box<std::error::Error>> {
//!
//! // Initialize with an API key
//! let client = Algorithmia::client("111112222233333444445555566")?;
//! let moving_avg = client.algo("timeseries/SimpleMovingAverage/0.1");
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = (vec![0,1,2,3,15,4,5,6,7], 3);
//! let result: Vec<f64> = moving_avg.pipe(&input)?.decode()?;
//! println!("Completed with result: {:?}", result);
//! # Ok(())
//! # }
//! ```

use crate::client::HttpClient;
use crate::error::{ApiErrorResponse, Error, ErrorKind, ResultExt};
use crate::Body;

use serde::de::DeserializeOwned;
use serde::de::Error as SerdeError;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

use base64;
use headers_ext::ContentType;
use mime::{self, Mime};
#[doc(hidden)]
pub use reqwest::Response;
use reqwest::Url;

use headers_ext::HeaderMapExt;
use http::header::HeaderMap;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Read, Write};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

static ALGORITHM_BASE_PATH: &'static str = "v1/algo";

/// Types that store either input or ouput to an algorithm
pub enum AlgoIo {
    /// Text input or output
    Text(String),
    /// Binary input or output
    Binary(Vec<u8>),
    /// JSON input or output
    Json(Value),
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

/// Successful API response that wraps the `AlgoIo` and its Metadata
pub struct AlgoResponse {
    /// Any metadata associated with the API response
    pub metadata: AlgoMetadata,
    /// The algorithm output decoded into an `AlgoIo` enum
    pub result: AlgoIo,
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

    /// Get the API Endpoint URL for this Algorithm
    pub fn to_url(&self) -> Result<Url, Error> {
        let path = format!("{}/{}", ALGORITHM_BASE_PATH, self.algo_uri.path);
        self.client
            .base_url
            .join(&path)
            .chain_err(|| ErrorKind::InvalidAlgoUri(path))
    }

    /// Get the Algorithmia algo URI for this Algorithm
    pub fn to_algo_uri(&self) -> &AlgoUri {
        &self.algo_uri
    }

    /// Execute an algorithm with the specified `input_data`.
    ///
    /// `input_data` can be any type which converts into `AlgoIo`,
    ///   including strings, byte slices, and any serializable type.
    ///   To create serializable objects for complex input, annotate your type
    ///   with `#[derive(Serialize)]` (see [serde.rs](http://serde.rs) for details).
    ///   If you want to send a raw, unparsed JSON string, use the `pipe_json` method instead.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566").unwrap();
    /// let moving_avg = client.algo("timeseries/SimpleMovingAverage/0.1");
    /// let input = (vec![0,1,2,3,15,4,5,6,7], 3);
    /// match moving_avg.pipe(&input) {
    ///     Ok(response) => println!("{}", response.into_json().unwrap()),
    ///     Err(err) => println!("ERROR: {}", err),
    /// };
    /// ```
    pub fn pipe<I>(&self, input_data: I) -> Result<AlgoResponse, Error>
    where
        I: Into<AlgoIo>,
    {
        let mut res = match input_data.into() {
            AlgoIo::Text(text) => self.pipe_as(text, mime::TEXT_PLAIN)?,
            AlgoIo::Json(json) => {
                let encoded = serde_json::to_vec(&json)
                    .chain_err(|| ErrorKind::EncodeJson("algorithm input"))?;
                self.pipe_as(encoded, mime::APPLICATION_JSON)?
            }
            AlgoIo::Binary(bytes) => self.pipe_as(bytes, mime::APPLICATION_OCTET_STREAM)?,
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
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// let client = Algorithmia::client("111112222233333444445555566")?;
    /// let minmax  = client.algo("codeb34v3r/FindMinMax/0.1");
    ///
    /// let output = match minmax.pipe_json("[2,3,4]") {
    ///    Ok(response) => response.into_json().unwrap(),
    ///    Err(err) => panic!("{}", err),
    /// };
    /// # Ok(())
    /// # }
    pub fn pipe_json(&self, json_input: &str) -> Result<AlgoResponse, Error> {
        let mut res = self.pipe_as(json_input.to_owned(), mime::APPLICATION_JSON)?;

        let mut res_json = String::new();
        res.read_to_string(&mut res_json)
            .chain_err(|| "failed to read algorithm response")?;
        res_json.parse()
    }

    #[doc(hidden)]
    pub fn pipe_as<B>(&self, input_data: B, content_type: Mime) -> Result<Response, Error>
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
        let mut headers = HeaderMap::new();
        headers.typed_insert(ContentType::from(content_type));
        self.client
            .post(url)
            .headers(headers)
            .body(input_data)
            .send()
            .chain_err(|| ErrorKind::Http(format!("calling algorithm '{}'", self.algo_uri)))
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
    ///
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// let client = Algorithmia::client("111112222233333444445555566")?;
    /// client.algo("codeb34v3r/FindMinMax/0.1")
    ///     .timeout(3)
    ///     .pipe(vec![2,3,4])?;
    /// # Ok(())
    /// # }
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

impl AlgoIo {
    /// If the `AlgoIo` is text (or a valid JSON string), returns the associated text
    #[allow(match_same_arms)]
    pub fn as_string(&self) -> Option<&str> {
        match *self {
            AlgoIo::Text(ref text) => Some(&*text),
            AlgoIo::Json(ref json) => json.as_str(),
            _ => None,
        }
    }

    /// If the `AlgoIo` is Json (or JSON encodable text), returns the associated JSON string
    ///
    /// For `AlgoIo::Json`, this returns the borrowed `Json`.
    ///   For the `AlgoIo::Text` variant, the text is wrapped into an owned `Value::String`.
    pub fn as_json<'a>(&'a self) -> Option<Cow<'a, Value>> {
        match *self {
            AlgoIo::Text(ref text) => Some(Cow::Owned(Value::String(text.to_owned()))),
            AlgoIo::Json(ref json) => Some(Cow::Borrowed(json)),
            AlgoIo::Binary(_) => None,
        }
    }

    /// If the `AlgoIo` is binary, returns the associated byte slice
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match *self {
            AlgoIo::Text(_) | AlgoIo::Json(_) => None,
            AlgoIo::Binary(ref bytes) => Some(&*bytes),
        }
    }

    /// If the `AlgoIo` is valid JSON, decode it to a particular type
    ///
    pub fn decode<D: DeserializeOwned>(&self) -> Result<D, Error> {
        let res_json = self
            .as_json()
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
            AlgoIo::Text(text) => Some(text),
            AlgoIo::Json(Value::String(text)) => Some(text),
            _ => None,
        }
    }

    /// Read algorithm output as JSON `Value` (if JSON of text)
    pub fn into_json(self) -> Option<Value> {
        match self.result {
            AlgoIo::Json(json) => Some(json),
            AlgoIo::Text(text) => Some(Value::String(text)),
            _ => None,
        }
    }

    /// If the algorithm output is Binary, returns the associated byte slice
    pub fn into_bytes(self) -> Option<Vec<u8>> {
        match self.result {
            AlgoIo::Binary(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// If the algorithm output is JSON, decode it into a particular type
    pub fn decode<D>(self) -> Result<D, Error>
    where
        for<'de> D: Deserialize<'de>,
    {
        let ct = self.metadata.content_type.clone();
        let res_json = self
            .into_json()
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
        AlgoOptions {
            opts: HashMap::new(),
        }
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
        let mut data =
            Value::from_str(json_str).chain_err(|| ErrorKind::DecodeJson("algorithm response"))?;
        let metadata_value = data
            .as_object_mut()
            .and_then(|ref mut o| o.remove("metadata"))
            .ok_or_else(|| serde_json::Error::missing_field("metadata"))
            .chain_err(|| ErrorKind::DecodeJson("algorithm response"))?;
        let result_value = data
            .as_object_mut()
            .and_then(|ref mut o| o.remove("result"))
            .ok_or_else(|| serde_json::Error::missing_field("result"))
            .chain_err(|| ErrorKind::DecodeJson("algorithm response"))?;

        // Construct the AlgoIo object
        let metadata = serde_json::from_value::<AlgoMetadata>(metadata_value)
            .chain_err(|| ErrorKind::DecodeJson("algorithm response metadata"))?;
        let result = match (&*metadata.content_type, result_value) {
            ("void", _) => AlgoIo::Json(Value::Null),
            ("json", value) => AlgoIo::Json(value),
            ("text", value) => match value.as_str() {
                Some(text) => AlgoIo::Text(text.into()),
                None => return Err(ErrorKind::MismatchedContentType("text").into()),
            },
            ("binary", value) => match value.as_str() {
                Some(text) => {
                    let binary = base64::decode(text)
                        .chain_err(|| ErrorKind::DecodeBase64("algorithm response"))?;
                    AlgoIo::Binary(binary)
                }
                None => return Err(ErrorKind::MismatchedContentType("binary").into()),
            },
            (content_type, _) => {
                return Err(ErrorKind::InvalidContentType(content_type.into()).into());
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
            AlgoIo::Text(ref s) => f.write_str(s),
            AlgoIo::Json(ref s) => f.write_str(&s.to_string()),
            AlgoIo::Binary(ref bytes) => f.write_str(&String::from_utf8_lossy(bytes)),
        }
    }
}

impl Read for AlgoResponse {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        match self.result {
            AlgoIo::Text(ref s) => buf.write(s.as_bytes()),
            AlgoIo::Json(ref s) => buf.write(s.to_string().as_bytes()),
            AlgoIo::Binary(ref bytes) => buf.write(bytes),
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
        AlgoUri {
            path: path.to_owned(),
        }
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

// AlgoIo Conversions
impl From<()> for AlgoIo {
    fn from(_unit: ()) -> Self {
        AlgoIo::Json(Value::Null)
    }
}

impl<'a> From<&'a str> for AlgoIo {
    fn from(text: &'a str) -> Self {
        AlgoIo::Text(text.to_owned())
    }
}

impl<'a> From<&'a [u8]> for AlgoIo {
    fn from(bytes: &'a [u8]) -> Self {
        AlgoIo::Binary(bytes.to_owned())
    }
}

impl From<String> for AlgoIo {
    fn from(text: String) -> Self {
        AlgoIo::Text(text.to_owned())
    }
}

impl From<Vec<u8>> for AlgoIo {
    fn from(bytes: Vec<u8>) -> Self {
        AlgoIo::Binary(bytes)
    }
}

impl From<Value> for AlgoIo {
    fn from(json: Value) -> Self {
        AlgoIo::Json(json)
    }
}

impl<'a, S: Serialize> From<&'a S> for AlgoIo {
    fn from(object: &'a S) -> Self {
        AlgoIo::Json(serde_json::to_value(object).expect("Failed to serialize"))
    }
}

impl<S: Serialize> From<Box<S>> for AlgoIo {
    fn from(object: Box<S>) -> Self {
        AlgoIo::Json(serde_json::to_value(object).expect("Failed to serialize"))
    }
}

// Waiting for specialization to stabilize
#[cfg(feature = "nightly")]
impl<S: Serialize> From<S> for AlgoIo {
    default fn from(object: S) -> Self {
        AlgoIo::Json(serde_json::to_value(object).expect("Failed to serialize"))
    }
}

macro_rules! impl_serialize_to_algo_io {
    ($t:ty) => {
        #[cfg(not(feature = "nightly"))]
        impl From<$t> for AlgoIo {
            fn from(t: $t) -> Self {
                AlgoIo::Json(serde_json::to_value(t).expect("Failed to serialize"))
            }
        }
    };
}

// Impl conversions for primitives until specialization stabilizes
impl_serialize_to_algo_io!(u32);
impl_serialize_to_algo_io!(i32);
impl_serialize_to_algo_io!(u64);
impl_serialize_to_algo_io!(i64);
impl_serialize_to_algo_io!(usize);
impl_serialize_to_algo_io!(isize);
impl_serialize_to_algo_io!(f32);
impl_serialize_to_algo_io!(f64);
impl_serialize_to_algo_io!(bool);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Algorithmia;

    fn mock_client() -> Algorithmia {
        Algorithmia::client("").unwrap()
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
    fn test_algo_with_sha_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo("anowell/Pinky/abcdef123456");
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
