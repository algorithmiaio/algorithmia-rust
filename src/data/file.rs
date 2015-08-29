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

use {Algorithmia, HttpClient};
use super::{parse_data_uri, HasDataPath, DeletedResult, XDataType, XErrorMessage, Body};
use std::io::{self, Read};
use error::{Error, ApiError};



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
    // pub meta: Metadata,
    data: Box<Read>,
}

impl Read for DataResponse {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}

/// Algorithmia data collection
pub struct DataFile {
    path: String,
    client: HttpClient,
}

impl HasDataPath for DataFile {
    fn new(client: HttpClient, path: &str) -> Self { DataFile { client: client, path: parse_data_uri(path).to_string() } }
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
            true => Algorithmia::decode_to_result(res_json),
            false => Err(Algorithmia::decode_to_error(res_json)),
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

        if res.status.is_success() {
            if let Some(data_type) = res.headers.get::<XDataType>() {
                if "file" != data_type.to_string() {
                    return Err(Error::DataTypeError(format!("Expected file, Received {}", data_type)));
                }
            }

            Ok(DataResponse{
                data: Box::new(res),
            })
        } else {
            let msg = match res.headers.get::<XErrorMessage>()  {
                Some(err_header) => format!("{}: {}", res.status, err_header),
                None => format!("{}", res.status),
            };

            Err(ApiError{message: msg, stacktrace: None}.into())
        }
    }


    /// Delete a file from a data collection
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
            true => Algorithmia::decode_to_result(res_json),
            false => Err(Algorithmia::decode_to_error(res_json)),
        }
    }

}