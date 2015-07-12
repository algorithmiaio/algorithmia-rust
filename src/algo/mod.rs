pub use self::algorithm::{Algorithm};
pub use self::result::{AlgoResult, JsonResult, AlgoOutput, AlgoMetadata};
pub use self::version::Version;

mod algorithm;
mod result;
mod version;