//! Manage data for algorithms
//!
//! Instantiate from the [`Algorithmia`](../struct.Algorithmia.html) struct

pub use self::dir::*;
pub use self::file::*;
pub use self::path::*;
pub use self::object::*;

use error::*;
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};
use std::time::{SystemTime, UNIX_EPOCH};
use std::ops::Deref;
use reqwest::header::{Headers, ContentLength, Date};

mod dir;
mod file;
mod path;
mod object;

static DATA_BASE_PATH: &'static str = "v1/connector";

mod header {
    header! { (XDataType, "X-Data-Type") => [String] }
    header! { (XErrorMessage, "X-Error-Message") => [String] }
}
use self::header::{XDataType, XErrorMessage};

/// Minimal representation of data type
pub enum DataType {
    File,
    Dir,
}

/// Data type wrapping the data item (including any metadata)
pub enum DataItem {
    File(DataFileItem),
    Dir(DataDirItem),
}

/// `DataFile` wrapper with metadata
pub struct DataFileItem {
    /// Size of file in bytes
    pub size: u64,
    /// Last modified timestamp
    pub last_modified: DateTime<Utc>,
    file: DataFile,
}

impl Deref for DataFileItem {
    type Target = DataFile;
    fn deref(&self) -> &DataFile {
        &self.file
    }
}

/// `DataDir` wrapper (currently no metadata)
pub struct DataDirItem {
    dir: DataDir,
}

impl Deref for DataDirItem {
    type Target = DataDir;
    fn deref(&self) -> &DataDir {
        &self.dir
    }
}

struct HeaderData {
    pub data_type: DataType,
    pub content_length: Option<u64>,
    pub last_modified: Option<DateTime<Utc>>,
}

fn parse_headers(headers: &Headers) -> Result<HeaderData> {
    if let Some(err_header) = headers.get::<XErrorMessage>() {
        return Err(
            ErrorKind::Api(ApiError {
                message: err_header.to_string(),
                stacktrace: None,
            }).into(),
        );
    };

    let data_type = match headers.get::<XDataType>() {
        Some(dt) if &*dt.to_string() == "directory" => DataType::Dir,
        Some(dt) if &*dt.to_string() == "file" => DataType::File,
        Some(dt) => return Err(ErrorKind::InvalidDataType(dt.to_string()).into()),
        None => return Err(ErrorKind::MissingDataType.into()),
    };

    let content_length = headers.get::<ContentLength>().map(|c| c.0);
    let last_modified = headers.get::<Date>().map(|d| {
        let time = SystemTime::from(d.0);
        let ts = time.duration_since(UNIX_EPOCH).expect("date header predates unix epoch");
        let naive_datetime = NaiveDateTime::from_timestamp(ts.as_secs() as i64, ts.subsec_nanos() as u32);
        Utc.from_utc_datetime(&naive_datetime)
    });

    Ok(HeaderData {
        data_type: data_type,
        content_length: content_length,
        last_modified: last_modified,
    })
}


fn parse_data_uri(data_uri: &str) -> String {
    match data_uri {
        p if p.contains("://") => p.split_terminator("://").collect::<Vec<_>>().join("/"),
        p if p.starts_with('/') => format!("data/{}", &p[1..]),
        p => format!("data/{}", p),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_data_uri;

    #[test]
    fn test_parse_protocol() {
        assert_eq!(parse_data_uri("data://"), "data");
        assert_eq!(parse_data_uri("data://foo"), "data/foo");
        assert_eq!(parse_data_uri("data://foo/"), "data/foo/");
        assert_eq!(parse_data_uri("data://foo/bar"), "data/foo/bar");
        assert_eq!(parse_data_uri("dropbox://"), "dropbox");
        assert_eq!(parse_data_uri("dropbox://foo"), "dropbox/foo");
        assert_eq!(parse_data_uri("dropbox://foo/"), "dropbox/foo/");
        assert_eq!(parse_data_uri("dropbox://foo/bar"), "dropbox/foo/bar");
    }

    #[test]
    fn test_parse_leading_slash() {
        assert_eq!(parse_data_uri("/foo"), "data/foo");
        assert_eq!(parse_data_uri("/foo/"), "data/foo/");
        assert_eq!(parse_data_uri("/foo/bar"), "data/foo/bar");
    }

    #[test]
    fn test_parse_unprefixed() {
        assert_eq!(parse_data_uri("foo"), "data/foo");
        assert_eq!(parse_data_uri("foo/"), "data/foo/");
        assert_eq!(parse_data_uri("foo/bar"), "data/foo/bar");
    }
}
