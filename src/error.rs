//! Error types
use error_chain::Backtrace;
use reqwest;
use reqwest::StatusCode;
use serde_json;
use std::fmt::Display;
use std::{fmt, str};

/// Default error type for errors originating in algorithm code
const ALGORITHM_ERROR: &'static str = "AlgorithmError";

fn default_error_type() -> String {
    ALGORITHM_ERROR.into()
}

error_chain! {
    types {
        Error, ErrorKind, ResultExt;
    }

    errors {
        // Error from the Algorithmia API (may be from the algorithm)
        Api(err: ApiError) {
            display("{}", err)
        }

        // Http errors calling the API
        Http(context: String) {
            display("HTTP error {}", context)
        }

        // Base URL couldn't be parsed as a `Url`
        InvalidBaseUrl {
            display("unable to parse base URL")
        }

        // Invalid Data URI
        InvalidDataUri(uri: String) {
            display("invalid data URI '{}'", uri)
        }

        // Invalid Algorithm URI
        InvalidAlgoUri(uri: String) {
            display("invalid algorithm URI: {}", &uri)
        }

        // Error decoding JSON
        DecodeJson(item: &'static str) {
            display("failed to decode {} json", item)
        }

        // Error encoding JSON
        EncodeJson(item: &'static str) {
            display("failed to encode {} as json", item)
        }

        // Error decoding base64
        DecodeBase64(item: &'static str) {
            display("failed to decode {} as base64", item)
        }

        // I/O errors reading or writing data
        Io(context: String) {
            display("I/O error {}", context)
        }

        // API responded with unknown content type
        InvalidContentType(t: String) {
            display("invalid content type: '{}'", t)
        }

        // Content was not valid for the specified content-type
        MismatchedContentType(expected: &'static str) {
            display("content did not match content type: '{}'", expected)
        }

        // Content type is not the expected content type
        UnexpectedContentType(expected: &'static str, actual: String) {
            display("expected content type '{}', received '{}'", expected, actual)
        }

        // Encountered 404 Not Found
        NotFound(url: reqwest::Url) {
            display("404 Not Found ({})", url)
        }

        // API response was missing a data type header
        MissingDataType {
            display("API response missing data type")
        }

        // API response included an unknown data type header
        InvalidDataType(t: String) {
            display("API responded with invalid data type: '{}'", t)
        }

        // API response included an unknown data type header
        InvalidApiKey {
            display("API key is invalid")
        }


        // API response included an unexpected data type header
        UnexpectedDataType(expected: &'static str, actual: String) {
            display("expected API response with data type '{}', received '{}'", expected, actual)
        }

        #[doc(hidden)]
        __DontMatchMe {}
    }
}

/// Error from the Algorithmia API (may be from the algorithm)
#[derive(Debug, Deserialize)]
pub struct ApiError {
    /// Error message returned from the Algorithmia API
    pub message: String,
    /// Error type
    #[serde(default = "default_error_type")]
    pub error_type: String,
    /// Stacktrace of algorithm exception/panic
    pub stacktrace: Option<String>,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.stacktrace {
            Some(ref trace) => write!(f, "{}\n{}", self.message, trace),
            None => write!(f, "{}", self.message),
        }
    }
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Error::from_kind(ErrorKind::Api(err))
    }
}

impl std::error::Error for ApiError {}

impl ApiError {
    /// Creates an ApiError - intended for creating ApiErrors from Rust algorithms
    ///
    /// ## Examples:
    ///
    /// ```
    /// use algorithmia::error::{ApiError, INPUT_ERROR};
    ///
    /// ApiError::new(INPUT_ERROR, "Input missing field 'url'");
    /// ```
    pub fn new<S: Into<String>>(error_type: S, message: S) -> ApiError {
        ApiError {
            error_type: error_type.into(),
            message: message.into(),
            stacktrace: Some(format!("{:?}", Backtrace::new())),
        }
    }

    pub(crate) fn from_json_or_status(json: &str, status: StatusCode) -> ApiError {
        match serde_json::from_str::<ApiErrorResponse>(json) {
            Ok(err_res) => err_res.error,
            Err(_) => ApiError::from(status.to_string()),
        }
    }
}

impl<S> From<S> for ApiError
where
    S: Into<String>,
{
    fn from(message: S) -> ApiError {
        ApiError {
            error_type: ALGORITHM_ERROR.into(),
            message: message.into(),
            stacktrace: Some(format!("{:?}", Backtrace::new())),
        }
    }
}

/// Struct for decoding Algorithmia API error responses
#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}

/// Helper to decode API responses into errors
#[doc(hidden)]
impl Error {
    pub fn from_json(json: &str) -> Error {
        let decoded_error = serde_json::from_str::<ApiErrorResponse>(json);
        match decoded_error.chain_err(|| ErrorKind::DecodeJson("api error response")) {
            Ok(err_res) => ErrorKind::Api(err_res.error).into(),
            Err(err) => err,
        }
    }
}
