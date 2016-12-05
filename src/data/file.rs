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
use data::{HasDataPath, DataType};
use std::io::{self, Read};
use ::Body;
use error::{self, ErrorKind, Result, ResultExt, ApiError};
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

        if res.status().is_success() {
            Ok(())
        } else {
            Err(error::decode(&res_json))
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
    pub fn get(&self) -> Result<FileData> {
        let url = self.to_url()?;
        let req = self.client.get(url);
        let res = req.send()
            .chain_err(|| ErrorKind::Http(format!("downloading file '{}'", self.to_data_uri())))?;
        let metadata = parse_headers(res.headers())?;

        if res.status().is_success() {
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
        } else {
            Err(ErrorKind::Api(ApiError {
                    message: res.status().to_string(),
                    stacktrace: None,
                })
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
    pub fn delete(&self) -> Result<()> {
        let url = self.to_url()?;
        let req = self.client.delete(url);
        let mut res = req.send()
            .chain_err(|| ErrorKind::Http(format!("deleting file '{}'", self.to_data_uri())))?;
        let mut res_json = String::new();
        res.read_to_string(&mut res_json)
            .chain_err(|| ErrorKind::Io(format!("deleting file '{}'", self.to_data_uri())))?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(error::decode(&res_json))
        }
    }
}
