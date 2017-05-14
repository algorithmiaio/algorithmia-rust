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
//! my_file.put("file_contents").unwrap();
//! ```

use chrono::{DateTime, UTC, TimeZone};
use client::HttpClient;
use reqwest::StatusCode;
use data::{HasDataPath, DataType};
use std::io::{self, Read};
use Body;
use error::{self, Error, ErrorKind, Result, ResultExt, ApiError};
use super::{parse_headers, parse_data_uri};


/// Response and reader when downloading a `DataFile`
pub struct FileData {
    /// Size of file in bytes
    pub size: u64,
    /// Last modified timestamp
    pub last_modified: DateTime<UTC>,
    data: Box<Read>,
}

impl Read for FileData {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}

impl FileData {
    /// Reads the result into a byte vector
    ///
    /// This is a convenience wrapper around `Read::read_to_end`
    /// that allocates once with capacity of `self.size`.
    pub fn into_bytes(mut self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(self.size as usize);
        self.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    /// Reads the result into a `String`
    ///
    /// This is a convenience wrapper around `Read::read_to_string`
    /// that allocates once with capacity of `self.size`.
    pub fn into_string(mut self) -> io::Result<String> {
        let mut text = String::with_capacity(self.size as usize);
        self.read_to_string(&mut text)?;
        Ok(text)
    }
}

/// Algorithmia data file
pub struct DataFile {
    path: String,
    client: HttpClient,
}

impl HasDataPath for DataFile {
    #[doc(hidden)]
    fn new(client: HttpClient, path: &str) -> Self {
        DataFile {
            client: client,
            path: parse_data_uri(path).to_string(),
        }
    }
    #[doc(hidden)]
    fn path(&self) -> &str {
        &self.path
    }
    #[doc(hidden)]
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
    /// # use std::fs::File;
    /// let client = Algorithmia::client("111112222233333444445555566");
    ///
    /// client.file(".my/my_dir/string.txt").put("file_contents").unwrap();
    /// client.file(".my/my_dir/bytes.txt").put("file_contents".as_bytes()).unwrap();
    ///
    /// let file = File::open("/path/to/file.jpg").unwrap();
    /// client.file(".my/my_dir/file.jpg").put(file).unwrap();
    /// ```
    pub fn put<B>(&self, body: B) -> Result<()>
        where B: Into<Body>
    {
        let url = self.to_url()?;
        let req = self.client.put(url).body(body);

        let mut res = req.send()
            .chain_err(|| ErrorKind::Http(format!("writing file '{}'", self.to_data_uri())))?;
        let mut res_json = String::new();
        res.read_to_string(&mut res_json)
            .chain_err(|| ErrorKind::Io(format!("writing file '{}'", self.to_data_uri())))?;

        match *res.status() {
            status if status.is_success() => Ok(()),
            StatusCode::NotFound => Err(ErrorKind::NotFound(self.to_url().unwrap()).into()),
            status => {
                let api_error = ApiError {
                    message: status.to_string(),
                    stacktrace: None,
                };
                Err(Error::from(ErrorKind::Api(api_error))).chain_err(|| error::decode(&res_json))
            }
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
    ///     match response.into_string() {
    ///       Ok(data) => println!("{}", data),
    ///       Err(err) => println!("IOError: {}", err),
    ///     }
    ///   },
    ///   Err(err) => println!("Error downloading file: {}", err),
    /// };
    /// ```
    pub fn get(&self) -> Result<FileData> {
        let url = self.to_url()?;
        let req = self.client.get(url);
        let res = req.send()
            .chain_err(|| ErrorKind::Http(format!("downloading file '{}'", self.to_data_uri())))?;

        match *res.status() {
            StatusCode::Ok => {
                let metadata = parse_headers(res.headers())?;
                match metadata.data_type {
                    DataType::File => (),
                    DataType::Dir => {
                        return Err(ErrorKind::UnexpectedDataType("file", "directory".to_string())
                            .into());
                    }
                }

                Ok(FileData {
                    size: metadata.content_length.unwrap_or(0),
                    last_modified: metadata.last_modified
                        .unwrap_or_else(|| UTC.ymd(2015, 3, 14).and_hms(8, 0, 0)),
                    data: Box::new(res),
                })
            }
            StatusCode::NotFound => Err(Error::from(ErrorKind::NotFound(self.to_url().unwrap()))),
            status => {
                Err(ErrorKind::Api(ApiError {
                        message: status.to_string(),
                        stacktrace: None,
                    })
                    .into())
            }
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
    pub fn delete(&self) -> Result<()> {
        let url = self.to_url()?;
        let req = self.client.delete(url);
        let mut res = req.send()
            .chain_err(|| ErrorKind::Http(format!("deleting file '{}'", self.to_data_uri())))?;
        let mut res_json = String::new();
        res.read_to_string(&mut res_json)
            .chain_err(|| ErrorKind::Io(format!("deleting file '{}'", self.to_data_uri())))?;

        match *res.status() {
            status if status.is_success() => Ok(()),
            StatusCode::NotFound => Err(ErrorKind::NotFound(self.to_url().unwrap()).into()),
            status => {
                let api_error = ApiError {
                    message: status.to_string(),
                    stacktrace: None,
                };
                Err(Error::from(ErrorKind::Api(api_error))).chain_err(|| error::decode(&res_json))
            }
        }
    }
}
