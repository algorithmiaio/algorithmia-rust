use ::error::{Error, Result};
use rustc_serialize::{Decodable, Encodable};
use rustc_serialize::json::{self, Json, DecoderError, EncoderError, ParserError};


quick_error! {
    #[derive(Debug)]
    pub enum JsonError {
        Parser(err: ParserError) {
            cause(err)
            description(err.description())
            display("{}", err)
        }
        Decoder(err: DecoderError) {
            cause(err)
            description(err.description())
            display("{}", err)
        }
        Encoder(err: EncoderError) {
            cause(err)
            description(err.description())
            display("{}", err)
        }
    }
}

impl From<DecoderError> for Error {
    fn from(err: DecoderError) -> Self {
        Error::Json(JsonError::Decoder(err))
    }
}

impl From<EncoderError> for Error {
    fn from(err: EncoderError) -> Self {
        Error::Json(JsonError::Encoder(err))
    }
}

impl From<ParserError> for Error {
    fn from(err: ParserError) -> Self {
        Error::Json(JsonError::Parser(err))
    }
}

pub fn decode_value<D: Decodable>(json: Json) -> Result<D> {
    let encoded = try!(json::encode(&json));
    decode_str(&encoded)
}

pub fn decode_str<D: Decodable>(json: &str) -> Result<D> {
    json::decode(json).map_err(Error::from)
}

pub fn value_from_str(json: &str) -> Result<Json> {
    Json::from_str(json).map_err(Error::from)
}

pub fn encode<E: Encodable>(value: E) -> Result<String> {
    json::encode(&value).map_err(Error::from)
}

pub fn value_as_str(json: &Json) -> Option<&str> {
    match *json {
        Json::String(ref text) => Some(&*text),
        _ => None,
    }
}

pub fn missing_field_error(field: &'static str) -> Error {
    DecoderError::MissingFieldError(field.to_string()).into()
}
