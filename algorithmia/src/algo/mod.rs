//! Interact with Algorithmia algorithms
//!
//! Instantiate from the [`Algorithmia`](../struct.Algorithmia.html) struct
pub use self::algorithm::*;

#[cfg(feature = "nightly")]
pub use self::entrypoint::*;

mod algorithm;

#[cfg(feature = "nightly")]
mod macros;

#[cfg(feature = "nightly")]
mod entrypoint;
