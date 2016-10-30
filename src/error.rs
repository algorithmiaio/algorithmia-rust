//! Error types
use std::{result, fmt, io, str};
use std::fmt::Display;
use ::json;
use base64;
use url;
use reqwest;


quick_error! {
    #[derive(Debug)]
    pub enum Error {

        Http(err: reqwest::Error) {
            from()
            from(e: url::ParseError) -> (reqwest::Error::from(e))
            cause(err)
            description(err.description())
            display("http error: {}", err)
        }

        Json(err: json::JsonError) {
            from()
            cause(err)
            description("json error")
            display("json error: {}", err)
        }

        Base64(err: base64::Base64Error) {
            from()
            cause(err)
            description("base64 error")
            display("base64 error: {}", err)
        }

        Io(err: io::Error) {
            from()
            cause(err)
            description("io error")
            display("io error: {}", err)
        }

        Utf8(err: str::Utf8Error) {
            from()
            cause(err)
            description("utf8 error")
            display("utf8 error: {}", err)
        }

        Api(err: ApiError) {
            from()
            description("api error")
            display("api error: {}", err)
        }

        InvalidContentType(t: String) {
            description("invalid content type")
            display("invalid content type: '{}'", t)
        }

        MismatchedContentType(expected: &'static str) {
            description("mismatched content type")
            display("content did not match content type: '{}'", expected)
        }

        UnexpectedContentType(expected: &'static str, actual: String) {
            description("unexpected content type")
            display("expected content type '{}', received '{}'", expected, actual)
        }

        MissingDataType {
            description("missing data type")
        }

        NotFound(url: reqwest::Url) {
            description("404 not found")
            display("404 not found: {}", url)
        }

        InvalidDataType(t: String) {
            description("invalid data type")
            display("invalid data type: '{}'", t)
        }

        UnexpectedDataType(expected: &'static str, actual: String) {
            description("unexpected data type")
            display("expected data type '{}', received '{}'", expected, actual)
        }

        InvalidDataPath(path: String) {
            description("invalid data path")
            display("invalid data path: '{}'", path)
        }

        UnsupportedInput {
            description("unsupported input type")
        }

    }
}

pub type Result<T> = result::Result<T, Error>;


#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable))]
#[derive(Debug)]
pub struct ApiError {
    pub message: String,
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
pub struct ApiErrorResponse {
    pub error: ApiError,
}


pub fn decode(json_str: &str) -> Error {
    match json::decode_str::<ApiErrorResponse>(json_str) {
        Ok(err_res) => err_res.error.into(),
        Err(err) => err,
    }
}
