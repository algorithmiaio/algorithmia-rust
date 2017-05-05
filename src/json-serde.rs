use serde_json::{self, Value};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::str::FromStr;
use serde::de::Error as SerdeError;

pub use serde_json::Error as JsonError;

pub fn decode_value<D: DeserializeOwned>(json: Value) -> Result<D, JsonError> {
    serde_json::from_value(json)
}

pub fn decode_str<'de, D: Deserialize<'de>>(json: &'de str) -> Result<D, JsonError> {
    serde_json::from_str(json)
}

pub fn value_from_str(json: &str) -> Result<Value, JsonError> {
    Value::from_str(json)
}

pub fn take_field(json: &mut Value, field: &str) -> Option<Value> {
    json.as_object_mut()
        .and_then(|ref mut o| o.remove(field))
}

// Could use `to_string`, but really just needs to return `Result<impl Into<Body>>`
//   so `to_vec` seems good
pub fn encode<S: Serialize>(value: S) -> Result<Vec<u8>, JsonError> {
    serde_json::to_vec(&value)
}

pub fn value_as_str(json: &Value) -> Option<&str> {
    match *json {
        #[cfg(feature="with-serde")] Value::String(ref text) => Some(&*text),
        #[cfg(feature="with-rustc-serialize")] Json::String(ref text) => Some(&*text),
        _ => None,
    }
}

pub fn missing_field_error(field: &'static str) -> JsonError {
    JsonError::missing_field(field)
}
