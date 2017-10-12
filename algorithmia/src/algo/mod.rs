//! Interact with Algorithmia algorithms
//!
//! Instantiate from the [`Algorithmia`](../struct.Algorithmia.html) struct
pub use self::algorithm::*;
pub use self::version::*;

#[cfg(feature = "nightly")]
pub use self::entrypoint::*;

mod algorithm;
mod version;

#[cfg(feature = "nightly")]
mod macros;

#[cfg(feature = "nightly")]
mod entrypoint;
