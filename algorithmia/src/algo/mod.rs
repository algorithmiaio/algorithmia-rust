//! Interact with Algorithmia algorithms
//!
//! Instantiate from the [`Algorithmia`](../struct.Algorithmia.html) struct
pub use self::algorithm::*;
pub use self::version::*;
pub use self::entrypoint::*;

mod algorithm;
mod version;
mod macros;
mod entrypoint;
