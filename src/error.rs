//! Error types
use std::{fmt, str};
use std::fmt::Display;
use ::json;
use reqwest;

error_chain! {
    errors {
        /// Error from the Algorithmia API (may be from the algorithm)
        Api(err: ApiError) {
            description("api error")
            display("api error - {}", err)
        }

        /// Http errors calling the API
        Http(context: String) {
            description("http error")
            display("http error {}", context)
        }

        /// Base URL couldn't be parsed as a `Url`
        InvalidBaseUrl {
            description("invalid base url")
        }

        /// Invalid Data URI
        InvalidDataUri(uri: String) {
            description("invalid data uri")
            display("invalid data uri '{}'", uri)
        }

        /// Invalid Algorithm URI
        InvalidAlgoUri(uri: String) {
            description("invalid algorithm uri")
            display("invalid algorithm uri: {}", &uri)
        }

        /// Error decoding JSON
        DecodeJson(item: &'static str) {
            description("json decode error")
            display("failed to decode {} json", item)
        }

        /// Error encoding JSON
        EncodeJson(item: &'static str) {
            description("json encode error")
            display("failed to encode {} as json", item)
        }

        /// Error decoding base64
        DecodeBase64(item: &'static str) {
            description("base64 error")
            display("failed to decode {} as base64", item)
        }

        /// I/O errors reading or writing data
        Io(context: String) {
            description("io error")
            display("io error {}", context)
        }

        /// API responded with unknown content type
        InvalidContentType(t: String) {
            description("invalid content type")
            display("invalid content type: '{}'", t)
        }

        /// Content was not valid for the specified content-type
        MismatchedContentType(expected: &'static str) {
            description("mismatched content type")
            display("content did not match content type: '{}'", expected)
        }

        /// Content type is not the expected content type
        UnexpectedContentType(expected: &'static str, actual: String) {
            description("unexpected content type")
            display("expected content type '{}', received '{}'", expected, actual)
        }

        /// Encountered 404 Not Found
        NotFound(url: reqwest::Url) {
            description("404 not found")
            display("404 not found: {}", url)
        }

        /// API response was missing a data type header
        MissingDataType {
            description("api response missing data type")
        }

        /// API response included an unknown data type header
        InvalidDataType(t: String) {
            description("invalid data type")
            display("api responded with invalid data type: '{}'", t)
        }

        /// API response included an unexpected data type header
        UnexpectedDataType(expected: &'static str, actual: String) {
            description("unexpected data type")
            display("expected api response with data type '{}', received '{}'", expected, actual)
        }

        /// Entrypoint not defined for input type
        UnsupportedInput {
            description("unsupported input type")
        }

        #[doc(hidden)]
        __DontMatchMe {}
    }
}


/// Error from the Algorithmia API (may be from the algorithm)
#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable))]
#[derive(Debug)]
pub struct ApiError {
    /// Error message returned from the Algorithmia API
    pub message: String,
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

/// Struct for decoding Algorithmia API error responses
#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable))]
#[derive(Debug)]
#[doc(hidden)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}


/// Helper to decode API responses into errors
pub fn decode(json_str: &str) -> Error {
    let decoded_error = json::decode_str::<ApiErrorResponse>(json_str);
    match decoded_error.chain_err(|| ErrorKind::DecodeJson("api error response")) {
        Ok(err_res) => ErrorKind::Api(err_res.error).into(),
        Err(err) => err,
    }
}
