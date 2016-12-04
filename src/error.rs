//! Error types
use std::{fmt, str};
use std::fmt::Display;
use ::json;
use reqwest;

error_chain! {
    errors {
        Http(context: String) {
            description("http error")
            display("http error {}", context)
        }

        InvalidBaseUrl {
            description("invalid base url")
        }

        InvalidUrlPath(path: String) {
            description("invalid path")
            display("invalid url path '{}'", path)
        }

        InvalidAlgoUri(uri: String) {
            description("invalid algorithm uri")
            display("invalid algorithm uri: {}", &uri)
        }

        DecodeJson(item: &'static str) {
            description("json decode error")
            display("failed to decode {} json", item)
        }

        EncodeJson(item: &'static str) {
            description("json encode error")
            display("failed to encode {} as json", item)
        }

        DecodeBase64(item: &'static str) {
            description("base64 error")
            display("failed to decode {} as base64", item)
        }

        Io(context: String) {
            description("io error")
            display("io error {}", context)
        }

        Api(err: ApiError) {
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

        #[doc(hidden)]
        __DontMatchMe {}
    }
}


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
    let decoded_error = json::decode_str::<ApiErrorResponse>(json_str);
    match decoded_error.chain_err(|| ErrorKind::DecodeJson("api error response")) {
            Ok(err_res) => ErrorKind::Api(err_res.error).into(),
            Err(err) => err,
    }
}
