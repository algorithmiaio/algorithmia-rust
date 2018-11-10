//! Algorithmia entrypoint for authoring Rust-based algorithms
//!
//! This module contains the traits that are used for defining the entrypoint
//! of a rust-based algorithm on the Algorithmia platform.
//!
//! Rust algorithms are loaded by instantiating an `Algo` type via `Algo::default()`
//! and each API request invokes the `apply` method on that instance.
//! While it is possible to manually define the `Algo` type and implement
//! either `EntryPoint` or `DecodedEntryPoint`, the `#[entrypoint]`
//! attribute will generate these implementations for you based on the signature
//! of the function (or impl) that it is attached to.
//!
//! ## `#[entrypoint]` Usage
//!
//! Just annotate a function with `#[entrypoint]`
//!
//! ```no_compile
//! #[entrypoint]
//! fn apply(name: String) -> Result<String, String> {
//!     Ok(format!("Hello, {}.", name))
//! }
//! ```
//!
//! This will generate the algorithm entrypoint as long the function
//! signature specifices supported input and output types:
//!
//! **Supported input types**:
//! - String types (e.g. `&str` and `String`)
//! - Byte types (e.g. `&[u8]` and `Vec<u8>`)
//! - Json `&Value` type
//! - `AlgoInput` enum type (for matching on text, json, and binary input)
//! - Any type that implements `serde::Deserialize` (e.g. `#[derive(Deserialize)]`)
//!
//! **Supported output (`Ok` variant of return value)**:
//! - `String`, `Vec<u8>`, `Value`, `AlgoOutput`
//! - Any type that implements `serde::Serialize` (e.g. `#[derive(Serialize)]`)
//!
//! **Supported error types (`Err` variant of return value)**:
//! - Any type with a conversion to `Box<std::error::Error>`. This includes `String` and basically any type that implements the `Error` trait.
//!
//!
//! ## Automatic JSON serialization/deserialization
//!
//! Since `#[entrypoint]` supports `Deserialize` input and `Serialize` output, this example will
//! accept a JSON array and return a JSON number:
//!
//! ```no_compile
//! #[entrypoint]
//! fn start(names: Vec<String>) -> Result<usize, String> {
//!    Ok(name.len())
//! }
//! ```
//!
//! To use your own custom types as input and output, simply implement `Deserialize` and
//! `Serialize` respectively.
//!
//! ```no_compile
//! #[derive(Deserialize)
//! struct Input { titles: Vec<String> }
//!
//! #[derive(Serialize)
//! struct Output { count: u32 }
//!
//! #[entrypoint]
//! fn start(input: Input) -> Result<Output, String> {
//!    Ok(Output{ count: input.titles.len() })
//! }
//! ```
//!
//! ## Preloading (advanced usage)
//!
//! If your algorithm has a preload step that doesn't vary with user input (e.g. loading a model),
//! you can create a type that implements `Default`, and use a method on that type as your
//! entrypoint (the `#[entrypoint]` annotation goes on the impl of your type. Multiple API calls in
//! succession from a single user will only instantiate the type once, but call `apply` multiple
//! times:
//!
//! ```no_compile
//! #[derive(Deserialize)
//! struct Input { titles: Vec<String>, max: u32 }
//!
//! #[derive(Serialize)
//! struct Output { titles: Vec<String> }
//!
//! struct App { model: Vec<u8> }
//!
//! #[entrypoint]
//! impl App {
//!     fn apply(&self, input: Input) -> Result<Output, String> {
//!         unimplemented!();
//!     }
//! }
//!
//! impl Default for App {
//!     fn default() -> Self {
//!         App { model: load_model() }
//!     }
//! }
//! ```

use crate::algo::{AlgoInput, AlgoOutput};
use std::error::Error as StdError;
use crate::error::{ErrorType, ApiError, ResultExt};
use serde_json;

use serde::de::DeserializeOwned;
use serde_json::Value;

// #[cfg(feature = "entrypoint")]
// #[doc(hidden)]
// pub use algorithmia_entrypoint::entrypoint;

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
///     fn apply_decoded(&mut self, input: Self::Input) -> Result<AlgoOutput, Box<Error>> {
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
    fn apply_decoded(&mut self, input: Self::Input) -> Result<AlgoOutput, Box<StdError>>;
}

impl<T> EntryPoint for T
where
    T: DecodedEntryPoint,
{
    fn apply(&mut self, input: AlgoInput) -> Result<AlgoOutput, Box<StdError>> {
        match input.as_json() {
            Some(obj) => {
                let decoded =
                    serde_json::from_value(obj.into_owned())
                        .chain_err(|| "failed to parse input as JSON into the expected type")?;
                self.apply_decoded(decoded)
            }
            None => Err(ApiError::new(ErrorType::Input, "Failed to parse input as JSON").into()),
        }
    }
}

/// Implementing an algorithm involves overriding at least one of these methods
pub trait EntryPoint: Default {
    #[allow(unused_variables)]
    /// Override to handle string input
    fn apply_str(&mut self, text: &str) -> Result<AlgoOutput, Box<StdError>> {
        Err(ApiError::new(ErrorType::Unsupported, "String input is not supported").into())
    }

    #[allow(unused_variables)]
    /// Override to handle JSON input (see also [`DecodedEntryPoint`](trait.DecodedEntryPoint.html))
    fn apply_json(&mut self, json: &Value) -> Result<AlgoOutput, Box<StdError>> {
        Err(ApiError::new(ErrorType::Unsupported, "JSON input is not supported").into())
    }

    #[allow(unused_variables)]
    /// Override to handle binary input
    fn apply_bytes(&mut self, bytes: &[u8]) -> Result<AlgoOutput, Box<StdError>> {
        Err(ApiError::new(ErrorType::Unsupported, "Binary input is not supported").into())
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
    fn apply(&mut self, input: AlgoInput) -> Result<AlgoOutput, Box<StdError>> {
        match input {
            AlgoInput::Text(ref text) => {
                match self.apply_str(text) {
                    Err(err) => {
                        match err.downcast_ref::<ApiError>().map(|err| err.error_type) {
                            Some(ErrorType::Unsupported) => {
                                match input.as_json() {
                                    Some(json) => self.apply_json(&json),
                                    None => Err(err.into()),
                                }
                            }
                            _ => Err(err.into()),
                        }
                    }
                    ret => ret,
                }
            }
            AlgoInput::Json(ref json) => {
                match self.apply_json(json) {
                    Err(err) => {
                        match err.downcast_ref::<ApiError>().map(|err| err.error_type) {
                            Some(ErrorType::Unsupported) => {
                                match input.as_string() {
                                    Some(text) => self.apply_str(text),
                                    None => Err(err.into()),
                                }
                            }
                            _ => Err(err.into()),
                        }
                    }
                    ret => ret,
                }
            }
            AlgoInput::Binary(ref bytes) => self.apply_bytes(bytes),
        }
    }
}
