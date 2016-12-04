use rustc_serialize::{Decodable, Encodable};
use rustc_serialize::json::{self, Json, DecoderError, EncoderError, ParserError};

pub fn decode_value<D: Decodable>(json: Json) -> Result<D, DecoderError> {
    let encoded = json::encode(&json).unwrap();
    decode_str(&encoded)
}

pub fn decode_str<D: Decodable>(json: &str) -> Result<D, DecoderError> {
    json::decode(json)
}

pub fn value_from_str(json: &str) -> Result<Json, ParserError> {
    Json::from_str(json)
}

pub fn take_field(json: &mut Json, field: &str) -> Option<Json> {
    json.as_object_mut()
        .and_then(|ref mut o| o.remove(field))
}

pub fn encode<E: Encodable>(value: E) -> Result<String, EncoderError> {
    json::encode(&value)
}

pub fn value_as_str(json: &Json) -> Option<&str> {
    match *json {
        Json::String(ref text) => Some(&*text),
        _ => None,
    }
}

pub fn missing_field_error(field: &'static str) -> DecoderError {
    DecoderError::MissingFieldError(field.to_string())
}
