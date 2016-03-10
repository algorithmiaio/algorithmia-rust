pub use data::dir::{DataDir, DirEntry, DataAcl, ReadAcl};
pub use self::file::DataFile;
pub use self::path::{DataPath, HasDataPath};

pub mod dir;
pub mod file;
pub mod path;

static DATA_BASE_PATH: &'static str = "v1/data";

header! { (XDataType, "X-Data-Type") => [String] }
header! { (XErrorMessage, "X-Error-Message") => [String] }

pub enum DataType {
    File,
    Dir,
}

pub enum DataObject {
    File(DataFile),
    Dir(DataDir),
}

// Shared by results for deleting both files and directories
#[derive(RustcDecodable, Debug)]
pub struct DeletedResult {
    pub deleted: u64,
}

pub fn parse_data_uri(data_uri: &str) -> &str {
    match data_uri {
        p if p.starts_with("data://") => &p[7..],
        p if p.starts_with("/") => &p[1..],
        p => p,
    }
}

