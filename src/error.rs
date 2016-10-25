//! Error types
use std::error::Error as StdError;
use std::{fmt, io, str, string};
use base64;
use serde_json;
use hyper;


/// Errors that may be returned by this library
#[derive(Debug)]
pub enum Error {
    ApiError(ApiError),
    ContentTypeError(String),
    DataTypeError(String),
    DataPathError(String),
    HttpError(hyper::error::Error),
    JsonError(serde_json::Error),
    Base64Error(base64::Base64Error),
    IoError(io::Error),
    Utf8Error(str::Utf8Error),
    UnsupportedInput,
}

#[derive(Deserialize, Debug)]
pub struct ApiError {
    pub message: String,
    pub stacktrace: Option<String>,
}

/// Struct for decoding Algorithmia API error responses
#[derive(Deserialize, Debug)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}


pub fn decode(json_str: &str) -> Error {
    match serde_json::from_str::<ApiErrorResponse>(json_str) {
        Ok(err_res) => err_res.error.into(),
        Err(err) => Error::JsonError(err),
    }
}

impl StdError for ApiError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ApiError(ref e) => e.description(),
            Error::ContentTypeError(ref msg) => &msg,
            Error::DataTypeError(ref msg) => &msg,
            Error::DataPathError(ref msg) => &msg,
            Error::HttpError(ref e) => e.description(),
            Error::JsonError(ref e) => e.description(),
            Error::Base64Error(ref e) => e.description(),
            Error::IoError(ref e) => e.description(),
            Error::Utf8Error(ref e) => e.description(),
            Error::UnsupportedInput => "Unsupported input type",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::HttpError(ref e) => Some(e),
            Error::JsonError(ref e) => Some(e),
            Error::Base64Error(ref e) => Some(e),
            Error::IoError(ref e) => Some(e),
            Error::Utf8Error(ref e) => Some(e),
            _ => None,
        }
    }
}


// Implement Display trait
//

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}


// Implement From trait (used by try! macro)
//

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Error {
        Error::ApiError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<hyper::error::Error> for Error {
    fn from(err: hyper::error::Error) -> Error {
        Error::HttpError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(err)
    }
}

impl From<base64::Base64Error> for Error {
    fn from(err: base64::Base64Error) -> Error {
        Error::Base64Error(err)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::Utf8Error(err.utf8_error())
    }
}
