use std::error::Error as StdError;
use std::{fmt, io};
use rustc_serialize::json;
use hyper;


/// Errors that may be returned by this library
#[derive(Debug)]
pub enum Error {
    ApiError(ApiError),
    DataTypeError(String),
    DataPathError(String),
    HttpError(hyper::error::Error),
    DecoderError(json::DecoderError),
    DecoderErrorWithContext(json::DecoderError, String),
    EncoderError(json::EncoderError),
    IoError(io::Error),
}

#[derive(RustcDecodable, Debug)]
pub struct ApiError {
    pub message: String,
    pub stacktrace: Option<String>,
}

/// Struct for decoding Algorithmia API error responses
#[derive(RustcDecodable, Debug)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}


//
// Implement std::error::Error trait
//

impl StdError for ApiError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ApiError(ref e) => e.description(),
            Error::DataTypeError(ref msg) => &msg,
            Error::DataPathError(ref msg) => &msg,
            Error::HttpError(ref e) => e.description(),
            Error::DecoderError(ref e) => e.description(),
            Error::DecoderErrorWithContext(ref e, _) => e.description(),
            Error::EncoderError(ref e) => e.description(),
            Error::IoError(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::HttpError(ref e) => Some(e),
            Error::DecoderError(ref e) => Some(e),
            Error::DecoderErrorWithContext(ref e, _) => Some(e),
            Error::EncoderError(ref e) => Some(e),
            Error::IoError(ref e) => Some(e),
            _ => None,
        }
    }
}


//
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


//
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

impl From<json::DecoderError> for Error {
    fn from(err: json::DecoderError) -> Error {
        Error::DecoderError(err)
    }
}

impl From<json::EncoderError> for Error {
    fn from(err: json::EncoderError) -> Error {
        Error::EncoderError(err)
    }
}
