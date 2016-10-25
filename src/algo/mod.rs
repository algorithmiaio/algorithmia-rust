//! Interact with Algorithmia algorithms
//!
//! Instantiate from the [`Algorithmia`](../struct.Algorithmia.html) struct
pub use self::algorithm::*;
pub use self::version::*;

mod algorithm;
mod version;
