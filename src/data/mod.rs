//! Manage data for algorithms
//!
//! Instantiate from the [`Algorithmia`](../struct.Algorithmia.html) struct

pub use data::dir::{DataDir, DataAcl, ReadAcl};
pub use self::file::DataFile;
pub use self::path::{DataPath, HasDataPath};

use chrono::{DateTime, UTC, NaiveDateTime, TimeZone};
use std::ops::Deref;
use error::*;
use hyper::header::{Headers, ContentLength, Date};


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
    File(DataFileEntry),
    Dir(DataDirEntry),
}

pub struct DataFileEntry {
    pub size: u64,
    pub last_modified: DateTime<UTC>,
    file: DataFile,
}

impl Deref for DataFileEntry {
    type Target = DataFile;
    fn deref(&self) -> &DataFile {&self.file}
}

pub struct DataDirEntry {
    dir: DataDir,
}

impl Deref for DataDirEntry {
    type Target = DataDir;
    fn deref(&self) -> &DataDir {&self.dir}
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

pub struct HeaderData {
    pub data_type: DataType,
    pub content_length: Option<u64>,
    pub last_modified: Option<DateTime<UTC>>,
}

pub fn parse_headers(headers: &Headers) -> Result<HeaderData, Error> {
    if let Some(err_header) = headers.get::<XErrorMessage>()  {
        return Err(ApiError{ message: err_header.to_string(), stacktrace: None }.into())
    };

    let data_type = try!(match headers.get::<XDataType>() {
        Some(dt) if &*dt.to_string() == "directory" => Ok(DataType::Dir),
        Some(dt) if &*dt.to_string() == "file"  => Ok(DataType::File),
        Some(dt) => Err(Error::DataTypeError(format!("Unknown DataType: {}", dt.to_string()))),
        None => Err(Error::DataTypeError("Unspecified DataType".to_string())),
    });

    let content_length = headers.get::<ContentLength>().map(|c| c.0);
    let last_modified = headers.get::<Date>()
                            .map(|d| {
                                let hdt = d.0;
                                let ts = hdt.0.to_timespec();
                                let naive_datetime = NaiveDateTime::from_timestamp(ts.sec, ts.nsec as u32);
                                UTC.from_utc_datetime(&naive_datetime)
                            });

    Ok(HeaderData {
        data_type: data_type,
        content_length: content_length,
        last_modified: last_modified,
    })
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