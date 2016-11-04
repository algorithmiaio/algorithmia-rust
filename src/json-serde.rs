use ::error::{Error, Result};
use serde_json::{self, Value, ErrorCode as JsonErrorCode};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub use serde_json::Error as JsonError;

pub fn decode_value<D: Deserialize>(json: Value) -> Result<D> {
    serde_json::from_value(json).map_err(Error::from)
}

pub fn decode_str<D: Deserialize>(json: &str) -> Result<D> {
    serde_json::from_str(json).map_err(Error::from)
}

pub fn value_from_str(json: &str) -> Result<Value> {
    Value::from_str(json).map_err(Error::from)
}

// Could use `to_string`, but really just needs to return `Result<impl Into<Body>>`
//   so `to_vec` seems good
pub fn encode<S: Serialize>(value: S) -> Result<Vec<u8>> {
    serde_json::to_vec(&value).map_err(Error::from)
}

pub fn value_as_str(json: &Value) -> Option<&str> {
    match *json {
        #[cfg(feature="with-serde")] Value::String(ref text) => Some(&*text),
        #[cfg(feature="with-rustc-serialize")] Json::String(ref text) => Some(&*text),
        _ => None,
    }
}

pub fn missing_field_error(field: &'static str) -> Error {
    JsonError::Syntax(JsonErrorCode::MissingField(field), 0, 0).into()
}
