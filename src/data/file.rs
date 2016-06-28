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
use client::{Body, HttpClient};
use data::{self, HasDataPath, DataType, DeletedResult};
use std::io::{self, Read};
use error::{self, Error, ApiError};
use rustc_serialize::json;

/// Response when creating a file via the Data API
#[derive(RustcDecodable, Debug)]
pub struct FileAdded {
    pub result: String
}

/// Response when deleting a file from the Data API
#[derive(RustcDecodable, Debug)]
pub struct FileDeleted {
    pub result: DeletedResult
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
    fn new(client: HttpClient, path: &str) -> Self { DataFile { client: client, path: data::parse_data_uri(path).to_string() } }
    fn path(&self) -> &str { &self.path }
    fn client(&self) -> &HttpClient { &self.client }
}

impl DataFile  {
    /// Write to the Algorithmia Data API
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use std::io::{self, Read};
    /// let client = Algorithmia::client("111112222233333444445555566");
    ///
    /// client.clone().file(".my/my_dir/string.txt").put("file_contents");
    /// client.clone().file(".my/my_dir/bytes.txt").put("file_contents".as_bytes());
    ///
    /// let mut stdin = io::stdin();
    /// let data_file = client.clone().file(".my/my_dir/stdin.txt");
    /// data_file.put(&mut stdin);
    /// ```
    pub fn put<'a, B: Into<Body<'a>>>(&'a self, body: B) -> Result<FileAdded, Error> {
        let url = self.to_url();

        let req = self.client.put(url).body(body);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        match res.status.is_success() {
            true => json::decode(&res_json).map_err(|err| err.into()),
            false => Err(error::decode(&res_json)),
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
    pub fn get(&self) -> Result<DataResponse, Error>  {
        let url = self.to_url();

        let req = self.client.get(url);
        let res = try!(req.send());
        let metadata = try!(data::parse_headers(&res.headers));

        if res.status.is_success() {
            match metadata.data_type {
                DataType::File => (),
                DataType::Dir => {
                    return Err(Error::DataTypeError("Expected file, Received directory".to_string()));
                }
            }

            Ok(DataResponse{
                size: metadata.content_length.unwrap_or(0),
                last_modified: metadata.last_modified.unwrap_or_else(|| UTC.ymd(2015, 3, 14).and_hms(8, 0, 0)),
                data: Box::new(res),
            })
        } else {
            Err(ApiError{message: res.status.to_string(), stacktrace: None}.into())
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
        let url = self.to_url();

        let req = self.client.delete(url);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        match res.status.is_success() {
            true => json::decode(&res_json).map_err(|err| err.into()),
            false => Err(error::decode(&res_json)),
        }
    }

}