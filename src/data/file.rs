//! File module for managing Algorithmia Data Files
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Algorithmia;
//!
//! let client = Algorithmia::client("111112222233333444445555566");
//! let my_file = client.file(".my/my_dir/some_filename");
//!
//! my_file.put("file_contents");
//! ```

use chrono::{DateTime, UTC, TimeZone};
use client::HttpClient;
use data::*;
use std::io::{self, Read};
use ::{json, Body};
use error::{Error, ApiError, ApiErrorResponse};
use super::{parse_headers, parse_data_uri};


/// Response when creating a file via the Data API
#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable))]
#[derive(Debug)]
pub struct FileAdded {
    pub result: String,
}

/// Response when deleting a file from the Data API
#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable))]
#[derive(Debug)]
pub struct FileDeleted {
    pub result: DeletedResult,
}

pub struct DataResponse {
    pub size: u64,
    pub last_modified: DateTime<UTC>,
    data: Box<Read>,
}

impl Read for DataResponse {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}

/// Algorithmia data file
pub struct DataFile {
    path: String,
    client: HttpClient,
}

impl HasDataPath for DataFile {
    fn new(client: HttpClient, path: &str) -> Self {
        DataFile {
            client: client,
            path: parse_data_uri(path).to_string(),
        }
    }
    fn path(&self) -> &str {
        &self.path
    }
    fn client(&self) -> &HttpClient {
        &self.client
    }
}

impl DataFile {
    /// Write to the Algorithmia Data API
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use std::io::{self, Read};
    /// # use std::fs::File;
    /// let client = Algorithmia::client("111112222233333444445555566");
    ///
    /// client.clone().file(".my/my_dir/string.txt").put("file_contents");
    /// client.clone().file(".my/my_dir/bytes.txt").put("file_contents".as_bytes());
    ///
    /// let file = File::open("/path/to/file.jpg").unwrap();
    /// client.clone().file(".my/my_dir/file.jpg").put(file);
    /// ```
    pub fn put<B>(&self, body: B) -> Result<FileAdded, Error>
        where B: Into<Body>
    {
        let url = try!(self.to_url());
        let req = try!(self.client.put(url)).body(body);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        if res.status().is_success() {
            json::decode_str(&res_json).map_err(|err| err.into())
        } else {
            Err(try!(json::decode_str::<ApiErrorResponse>(&res_json)).error.into())
        }
    }


    /// Get a file from the Algorithmia Data API
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use std::io::Read;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_file = client.file(".my/my_dir/sample.txt");
    ///
    /// match my_file.get() {
    ///   Ok(mut response) => {
    ///     let mut data = String::new();
    ///     match response.read_to_string(&mut data) {
    ///       Ok(_) => println!("{}", data),
    ///       Err(err) => println!("IOError: {}", err),
    ///     }
    ///   },
    ///   Err(err) => println!("Error downloading file: {}", err),
    /// };
    /// ```
    pub fn get(&self) -> Result<DataResponse, Error> {
        let url = try!(self.to_url());
        let req = try!(self.client.get(url));
        let res = try!(req.send());
        let metadata = try!(parse_headers(res.headers()));

        if res.status().is_success() {
            match metadata.data_type {
                DataType::File => (),
                DataType::Dir => {
                    return Err(Error::UnexpectedDataType("file", "directory".to_string()));
                }
            }

            Ok(DataResponse {
                size: metadata.content_length.unwrap_or(0),
                last_modified: metadata.last_modified
                    .unwrap_or_else(|| UTC.ymd(2015, 3, 14).and_hms(8, 0, 0)),
                data: Box::new(res),
            })
        } else {
            Err(ApiError {
                    message: res.status().to_string(),
                    stacktrace: None,
                }
                .into())
        }
    }


    /// Delete a file from from the Algorithmia Data API
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_file = client.file(".my/my_dir/sample.txt");
    ///
    /// match my_file.delete() {
    ///   Ok(_) => println!("Successfully deleted file"),
    ///   Err(err) => println!("Error deleting file: {}", err),
    /// };
    /// ```
    pub fn delete(&self) -> Result<FileDeleted, Error> {
        let url = try!(self.to_url());
        let req = try!(self.client.delete(url));
        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        if res.status().is_success() {
            json::decode_str(&res_json).map_err(|err| err.into())
        } else {
            Err(try!(json::decode_str::<ApiErrorResponse>(&res_json)).error.into())
        }
    }
}
