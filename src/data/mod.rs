//! Manage data for algorithms
//!
//! Instantiate from the [`Algorithmia`](../struct.Algorithmia.html) struct

pub use self::dir::*;
pub use self::file::*;
pub use self::path::*;
pub use self::object::*;

use error::*;
use chrono::{DateTime, UTC, NaiveDateTime, TimeZone};
use std::ops::Deref;
use hyper::header::{Headers, ContentLength, Date};

mod dir;
mod file;
mod path;
mod object;

static DATA_BASE_PATH: &'static str = "v1/connector";

header! { (XDataType, "X-Data-Type") => [String] }
header! { (XErrorMessage, "X-Error-Message") => [String] }

pub enum DataType {
    File,
    Dir,
}

pub enum DataItem {
    File(DataFileItem),
    Dir(DataDirItem),
}

pub struct DataFileItem {
    pub size: u64,
    pub last_modified: DateTime<UTC>,
    file: DataFile,
}

impl Deref for DataFileItem {
    type Target = DataFile;
    fn deref(&self) -> &DataFile {&self.file}
}

pub struct DataDirItem {
    dir: DataDir,
}

impl Deref for DataDirItem {
    type Target = DataDir;
    fn deref(&self) -> &DataDir {&self.dir}
}

// Shared by results for deleting both files and directories
#[derive(RustcDecodable, Debug)]
pub struct DeletedResult {
    pub deleted: u64,
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
