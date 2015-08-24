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

use {Algorithmia, AlgorithmiaError, HttpClient};
use super::{DataObject, DeletedResult, XDataType, Body};
use std::io::{self, Read};
use std::ops::Deref;


pub type FileAddedResult = Result<FileAdded, AlgorithmiaError>;
pub type FileDeletedResult = Result<FileDeleted, AlgorithmiaError>;


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
    data_object: DataObject,
}

impl Deref for DataFile {
    type Target = DataObject;
    fn deref(&self) -> &DataObject {&self.data_object}
}

impl DataFile  {
    pub fn new(client: HttpClient, data_uri: &str) -> DataFile {
        DataFile {
            data_object: DataObject::new(client, data_uri),
        }
    }


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
    pub fn put<'a, B: Into<Body<'a>>>(&'a self, body: B) -> FileAddedResult {
        let url = self.to_url();

        let req = self.client.put(url).body(body);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Algorithmia::decode_to_result::<FileAdded>(res_json)
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
    ///       Err(e) => println!("IOError: {}", e),
    ///     }
    ///   },
    ///   Err(e) => println!("ERROR downloading file: {:?}", e),
    /// };
    /// ```
    pub fn get(&self) -> Result<DataResponse, AlgorithmiaError>  {
        let url = self.to_url();

        let req = self.client.get(url);

        let res = try!(req.send());

        if let Some(data_type) = res.headers.get::<XDataType>() {
            if "file" != data_type.to_string() {
                return Err(AlgorithmiaError::DataTypeError(format!("Expected file, Received {}", data_type)));
            }
        }

        Ok(DataResponse{
            data: Box::new(res),
        })
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
    ///   Err(e) => println!("ERROR deleting file: {:?}", e),
    /// };
    /// ```
    pub fn delete(&self) -> FileDeletedResult {
        let url = self.to_url();

        let req = self.client.delete(url);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Algorithmia::decode_to_result::<FileDeleted>(res_json)
    }

}