//! File module for managing Algorithmia Data Files
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Service;
//!
//! let service = Service::new("111112222233333444445555566");
//! let my_file = service.file(".my/my_dir/some_filename");
//!
//! my_file.put_bytes("file_contents".as_bytes());
//! ```

use {Service, AlgorithmiaError};
use super::{DataObject};
use std::io::Read;
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
    pub result: String
}

/// Algorithmia data collection
pub struct DataFile {
    data_object: DataObject,
}

impl Deref for DataFile {
    type Target = DataObject;
    fn deref(&self) -> &DataObject {&self.data_object}
}

impl DataFile {
    pub fn new(service: Service, data_uri: &str) -> DataFile {
        DataFile {
            data_object: DataObject::new(service, data_uri),
        }
    }


    /// Write a file (raw bytes) directly to a data collection
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// let service = Service::new("111112222233333444445555566");
    /// let my_file = service.file(".my/my_dir/sample.txt");
    ///
    /// match my_file.put_bytes("file_contents".as_bytes()) {
    ///   Ok(response) => println!("Successfully uploaded to: {}", response.result),
    ///   Err(e) => println!("ERROR uploading file: {:?}", e),
    /// };
    /// ```
    pub fn put_bytes(&self, input_data: &[u8]) -> FileAddedResult {
        let url = self.to_url();

        let ref mut api_client = self.service.api_client();
        let req = api_client.post(url).body(input_data);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Service::decode_to_result::<FileAdded>(res_json)
    }




    /// Delete a file from a data collection
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// let service = Service::new("111112222233333444445555566");
    /// let my_file = service.file(".my/my_dir/sample.txt");
    ///
    /// match my_file.delete() {
    ///   Ok(_) => println!("Successfully deleted file"),
    ///   Err(e) => println!("ERROR deleting file: {:?}", e),
    /// };
    /// ```
    pub fn delete(&self) -> FileDeletedResult {
        let url = self.to_url();

        let ref mut api_client = self.service.api_client();
        let req = api_client.delete(url);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Service::decode_to_result::<FileDeleted>(res_json)
    }

}