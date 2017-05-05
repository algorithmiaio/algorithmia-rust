use algo::{AlgoInput, AlgoOutput, JsonValue};
use std::error::Error as StdError;
use error::{Error, ErrorKind, ResultExt};
use json;

use serde::de::DeserializeOwned;

/// Alternate implementation for `EntryPoint`
///   that automatically decodes JSON input to the associate type
///
/// # Examples
/// ```no_run
/// # use algorithmia::prelude::*;
/// # use std::error::Error;
/// # #[derive(Default)]
/// # struct Algo;
/// impl DecodedEntryPoint for Algo {
///     // Expect input to be an array of 2 strings
///     type Input = (String, String);
///     fn apply_decoded(&self, input: Self::Input) -> Result<AlgoOutput, Box<Error>> {
///         let msg = format!("{} - {}", input.0, input.1);
///         Ok(msg.into())
///     }
/// }
/// ```
pub trait DecodedEntryPoint: Default {
    /// Specifies the type that the input will be automatically deserialized into
    type Input: DeserializeOwned;

    /// This method is an apply variant that will receive the decoded form of JSON input.
    ///   If decoding failed, a `DecoderError` will be returned before this method is invoked.
    #[allow(unused_variables)]
    fn apply_decoded(&self, input: Self::Input) -> Result<AlgoOutput, Box<StdError>>;
}

impl<T> EntryPoint for T
    where T: DecodedEntryPoint
{
    fn apply(&self, input: AlgoInput) -> Result<AlgoOutput, Box<StdError>> {
        match input.as_json() {
            Some(obj) => {
                let decoded = json::decode_value(obj.into_owned())
                    .chain_err(|| "failed to parse input as JSON into the expected type")?;
                self.apply_decoded(decoded)
            }
            None => Err(Error::from(ErrorKind::UnsupportedInput).into()),
        }
    }
}

/// Implementing an algorithm involves overriding at least one of these methods
pub trait EntryPoint: Default {
    #[allow(unused_variables)]
    /// Override to handle string input
    fn apply_str(&self, text: &str) -> Result<AlgoOutput, Box<StdError>> {
        Err(Error::from(ErrorKind::UnsupportedInput).into())
    }

    #[allow(unused_variables)]
    /// Override to handle JSON input (see also [`DecodedEntryPoint`](trait.DecodedEntryPoint.html))
    fn apply_json(&self, json: &JsonValue) -> Result<AlgoOutput, Box<StdError>> {
        Err(Error::from(ErrorKind::UnsupportedInput).into())
    }

    #[allow(unused_variables)]
    /// Override to handle binary input
    fn apply_bytes(&self, bytes: &[u8]) -> Result<AlgoOutput, Box<StdError>> {
        Err(Error::from(ErrorKind::UnsupportedInput).into())
    }

    /// The default implementation of this method calls
    /// `apply_str`, `apply_json`, or `apply_bytes` based on the input type.
    ///
    /// - `AlgoInput::Text` results in call to  `apply_str`
    /// - `AlgoInput::Json` results in call to  `apply_json`
    /// - `AlgoInput::Binary` results in call to  `apply_bytes`
    ///
    /// If that call returns anKind `UnsupportedInput` error, then this method
    ///   method will may attempt to coerce the input into another type
    ///   and attempt one more call:
    ///
    /// - `AlgoInput::Text` input will be JSON-encoded to call `apply_json`
    /// - `AlgoInput::Json` input will be parse to see it can call `apply_str`
    fn apply(&self, input: AlgoInput) -> Result<AlgoOutput, Box<StdError>> {
        match input {
            AlgoInput::Text(ref text) => {
                match self.apply_str(text) {
                    Err(err) => {
                        match err.downcast::<Error>().map(|err| *err) {
                            Ok(Error(ErrorKind::UnsupportedInput, _)) => {
                                match input.as_json() {
                                    Some(json) => self.apply_json(&json),
                                    None => Err(Error::from(ErrorKind::UnsupportedInput).into()),
                                }
                            }
                            Ok(err) => Err(err.into()),
                            Err(err) => Err(err.into()),
                        }
                    }
                    ret => ret,
                }
            }
            AlgoInput::Json(ref json) => {
                match self.apply_json(json) {
                    Err(err) => {
                        match err.downcast::<Error>().map(|err| *err) {
                            Ok(Error(ErrorKind::UnsupportedInput, _)) => {
                                match input.as_string() {
                                    Some(text) => self.apply_str(text),
                                    None => {
                                        Err(Error::from(ErrorKind::UnsupportedInput).into()).into()
                                    }
                                }
                            }
                            Ok(err) => Err(err.into()),
                            Err(err) => Err(err.into()),
                        }
                    }
                    ret => ret,
                }
            }
            AlgoInput::Binary(ref bytes) => self.apply_bytes(bytes),
        }
    }
}
