//! Manage data for algorithms
//!
//! Instantiate from the [`Algorithmia`](../struct.Algorithmia.html) struct
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_protocol() {
        assert_eq!(parse_data_uri("data://foo"), "foo");
        assert_eq!(parse_data_uri("data://foo/"), "foo/");
        assert_eq!(parse_data_uri("data://foo/bar"), "foo/bar");
    }

    #[test]
    fn test_parse_leading_slash() {
        assert_eq!(parse_data_uri("/foo"), "foo");
        assert_eq!(parse_data_uri("/foo/"), "foo/");
        assert_eq!(parse_data_uri("/foo/bar"), "foo/bar");
    }

    #[test]
    fn test_parse_unprefixed() {
        assert_eq!(parse_data_uri("foo"), "foo");
        assert_eq!(parse_data_uri("foo/"), "foo/");
        assert_eq!(parse_data_uri("foo/bar"), "foo/bar");
    }

}