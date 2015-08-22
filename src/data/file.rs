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
//! my_file.put_bytes("file_contents".as_bytes());
//! ```

use {Algorithmia, AlgorithmiaError};
use super::{DataObject, DeletedResult};
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
    pub result: DeletedResult
}

pub struct DataResponse {
    // pub meta: Metadata,
    pub data: Box<Read>,
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
    pub fn new(client: Algorithmia, data_uri: &str) -> DataFile {
        DataFile {
            data_object: DataObject::new(client, data_uri),
        }
    }


    /// Write a file (raw bytes) directly to the Algorithmia Data API
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_file = client.file(".my/my_dir/sample.txt");
    ///
    /// match my_file.put_bytes("file_contents".as_bytes()) {
    ///   Ok(response) => println!("Successfully uploaded to: {}", response.result),
    ///   Err(e) => println!("ERROR uploading file: {:?}", e),
    /// };
    /// ```
    // TODO: just use .put and whatever input_data type is used by .body
    pub fn put_bytes(&self, input_data: &[u8]) -> FileAddedResult {
        let url = self.to_url();

        let http_client = self.client.http_client();
        let req = http_client.put(url).body(input_data);

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
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_file = client.file(".my/my_dir/sample.txt");
    ///
    /// match my_file.get() {
    ///   Ok(response) => println!("{}", response.read_to_string()),
    ///   Err(e) => println!("ERROR downloading file: {:?}", e),
    /// };
    /// ```
    pub fn get(&self) -> Result<DataResponse, AlgorithmiaError>  {
        let url = self.to_url();

        let http_client = self.client.http_client();
        let req = http_client.get(url);

        let res = try!(req.send());

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

        let http_client = self.client.http_client();
        let req = http_client.delete(url);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Algorithmia::decode_to_result::<FileDeleted>(res_json)
    }

}