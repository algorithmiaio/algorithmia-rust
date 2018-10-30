//! Error types
use std::{fmt, str};
use std::fmt::Display;
use serde_json;
use reqwest;
use reqwest::StatusCode;
use error_chain::Backtrace;
use std::error::Error as StdError;


/// Error Types that the Algorithmia algorithm APIs can return
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorType {
    #[serde(rename="InputError")]
    Input,

    #[serde(rename="UnsupportedError")]
    Unsupported,

    #[serde(rename="InitializationError")]
    Initialization,

    #[serde(rename="OutOfMemoryError")]
    OutOfMemory,

    #[serde(rename="OutOfGpuMemoryError")]
    OutOfGpuMemory,

    #[serde(rename="LanguageError")]
    Language,

    #[serde(rename="TooLargeError")]
    TooLarge,

    #[serde(rename="ParsingError")]
    Parsing,

    #[serde(rename="EntityNotFoundError")]
    EntityNotFound,

    #[serde(rename="ThirdPartyCredentialError")]
    ThirdPartyCredential,

    #[serde(rename="AlgorithmError")]
    Algorithm,

    // Replace with #[non_exhaustive] after https://github.com/rust-lang/rust/issues/44109
    // This also causes unreachable pattern warning on deserialize since "UknownError" is listed twice
    #[doc(hidden)]
    #[serde(rename="AlgorithmError")]
    __DontMatchMe {}
}

fn unknown_error() -> ErrorType {
    ErrorType::Algorithm
}


error_chain! {
    foreign_links {
        // Error from the Algorithmia API (may be from the algorithm)
        Api(ApiError);
    }

    errors {
        // Http errors calling the API
        Http(context: String) {
            description("HTTP error")
            display("HTTP error {}", context)
        }

        // Base URL couldn't be parsed as a `Url`
        InvalidBaseUrl {
            description("invalid base URL")
        }

        // Invalid Data URI
        InvalidDataUri(uri: String) {
            description("invalid data URI")
            display("invalid data URI '{}'", uri)
        }

        // Invalid Algorithm URI
        InvalidAlgoUri(uri: String) {
            description("invalid algorithm URI")
            display("invalid algorithm URI: {}", &uri)
        }

        // Error decoding JSON
        DecodeJson(item: &'static str) {
            description("json decode error")
            display("failed to decode {} json", item)
        }

        // Error encoding JSON
        EncodeJson(item: &'static str) {
            description("json encode error")
            display("failed to encode {} as json", item)
        }

        // Error decoding base64
        DecodeBase64(item: &'static str) {
            description("base64 error")
            display("failed to decode {} as base64", item)
        }

        // I/O errors reading or writing data
        Io(context: String) {
            description("I/O error")
            display("I/O error {}", context)
        }

        // API responded with unknown content type
        InvalidContentType(t: String) {
            description("invalid content type")
            display("invalid content type: '{}'", t)
        }

        // Content was not valid for the specified content-type
        MismatchedContentType(expected: &'static str) {
            description("mismatched content type")
            display("content did not match content type: '{}'", expected)
        }

        // Content type is not the expected content type
        UnexpectedContentType(expected: &'static str, actual: String) {
            description("unexpected content type")
            display("expected content type '{}', received '{}'", expected, actual)
        }

        // Encountered 404 Not Found
        NotFound(url: reqwest::Url) {
            description("404 Not Found")
            display("404 Not Found ({})", url)
        }

        // API response was missing a data type header
        MissingDataType {
            description("API response missing data type")
        }

        // API response included an unknown data type header
        InvalidDataType(t: String) {
            description("invalid data type")
            display("API responded with invalid data type: '{}'", t)
        }

        // API response included an unknown data type header
        InvalidApiKey {
            description("invalid API key")
            display("API key is invalid")
        }


        // API response included an unexpected data type header
        UnexpectedDataType(expected: &'static str, actual: String) {
            description("unexpected data type")
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
    #[serde(default="unknown_error")]
    pub error_type: ErrorType,
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

impl StdError for ApiError {
    fn description(&self) -> &str {
        "API error"
    }
}

impl ApiError {
    /// Creates an ApiError - intended for creating ApiErrors from Rust algorithms
    ///
    /// ## Examples:
    ///
    /// ```
    /// use algorithmia::error::{ApiError, ErrorType};
    ///
    /// ApiError::new(ErrorType::Input, "Input missing field 'url'");
    /// ```
    pub fn new<S: Into<String>>(error_type: ErrorType, message: S) -> ApiError {
        ApiError {
            error_type,
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

impl <S> From<S> for ApiError where S: Into<String> {
    fn from(message: S) -> ApiError {
        ApiError {
            error_type: ErrorType::Algorithm,
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
