//! Collection module for managing Algorithmia Data Collections
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Service;
//! use std::fs::File;
//!
//! let service = Service::new("111112222233333444445555566");
//! let my_dir = service.collection("my_user/my_dir");
//!
//! my_dir.create();
//! my_dir.upload_file("/path/to/file");
//!
//! my_dir.write_file("some_filename", "file_contents".as_bytes());
//! ```

extern crate hyper;

use ::{Service, AlgorithmiaError, ApiErrorResponse};
use hyper::Url;
use rustc_serialize::{json, Decoder};
use std::io::Read;
use std::fs::File;
use std::path::Path;

static COLLECTION_BASE_PATH: &'static str = "data";

/// Algorithmia data collection
pub struct Collection<'a> {
    pub service: Service,
    pub path: &'a str,
}

pub type CollectionShowResult = Result<CollectionShow, AlgorithmiaError>;
pub type CollectionCreatedResult = Result<CollectionCreated, AlgorithmiaError>;
pub type CollectionDeletedResult = Result<CollectionDeleted, AlgorithmiaError>;
pub type CollectionFileAddedResult = Result<CollectionFileAdded, AlgorithmiaError>;
pub type CollectionFileDeletedResult = Result<CollectionFileDeleted, AlgorithmiaError>;


/// Permissions for a data collection
#[derive(RustcDecodable, Debug)]
pub struct CollectionAcl {
    /// Readable by world
    pub read_w: bool,
    /// Readable by group
    pub read_g: bool,
    /// Readable by user
    pub read_u: bool,
    /// Readable by user's algorithms regardless who runs them
    pub read_a: bool,
}

/// Response when creating a new collection
#[derive(RustcDecodable, Debug)]
pub struct CollectionCreated {
    pub collection_id: u32,
    pub object_id: String,
    pub collection_name: String,
    pub username: String,
    pub acl: CollectionAcl,
}

/// Response when deleting a new collection
#[derive(RustcDecodable, Debug)]
pub struct CollectionDeleted {
    pub unknown: String,
}


/// Response when querying an existing collection
#[derive(RustcDecodable, Debug)]
pub struct CollectionShow {
    pub username: String,
    pub collection_name: String,
    pub files: Vec<String>,
}

/// Response when adding a file to a collection
#[derive(RustcDecodable, Debug)]
pub struct CollectionFileAdded {
    pub result: String
}

/// Response when deleting a file to a collection
#[derive(RustcDecodable, Debug)]
pub struct CollectionFileDeleted {
    pub result: String
}

impl<'a> Collection<'a> {

    /// Get the parent path of a collection (i.e. unix `dirname`)
    ///
    /// ```
    /// # use algorithmia::Service;
    /// # let service = Service::new("111112222233333444445555566");
    /// let my_dir = service.collection("my_user/my_dir");
    /// assert_eq!(my_dir.parent(), "my_user");
    /// ```
    pub fn parent(&self) -> &'a str {
        match self.path.rsplitn(2, "/").nth(1) {
            Some(path) => path,
            None => "/"
        }
    }

    /// Get the basename of the collection (i.e. unix `basename`)
    ///
    /// ```
    /// # use algorithmia::Service;
    /// # let service = Service::new("111112222233333444445555566");
    /// let my_dir = service.collection("my_user/my_dir");
    /// assert_eq!(my_dir.basename(), "my_dir");
    /// ```
    pub fn basename(&self) -> &'a str {
        match self.path.rsplitn(2, "/").nth(0) {
            Some(path) => path,
            None => "/"
        }
    }


    /// Get the API Endpoint URL for a particular collection
    pub fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}", Service::get_api(), COLLECTION_BASE_PATH, self.path);
        Url::parse(&url_string).unwrap()
    }

    /// Display collection details if it exists
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// let service = Service::new("111112222233333444445555566");
    /// let my_dir = service.collection("my_user/my_dir");
    /// match my_dir.show() {
    ///   Ok(dir) => println!("Files: {}", dir.files.connect(", ")),
    ///   Err(e) => println!("ERROR: {:?}", e),
    /// };
    /// ```
    pub fn show(&'a self) -> CollectionShowResult {
        let ref mut api_client = self.service.api_client();
        let req = api_client.get(self.to_url());

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        match json::decode::<CollectionShow>(&res_json) {
            Ok(result) => Ok(result),
            Err(why) => match json::decode::<ApiErrorResponse>(&res_json) {
                Ok(api_error) => Err(AlgorithmiaError::ApiError(api_error.error)),
                Err(_) => Err(AlgorithmiaError::DecoderErrorWithContext(why, res_json)),
            }
        }
    }

    /// Create a collection
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// let service = Service::new("111112222233333444445555566");
    /// let my_dir = service.collection("my_user/my_dir");
    /// match my_dir.create() {
    ///   Ok(_) => println!("Successfully created collection"),
    ///   Err(e) => println!("ERROR creating collection: {:?}", e),
    /// };
    /// ```
    pub fn create(&'a self) -> CollectionCreatedResult {
        // Construct URL
        let url_string = format!("{}/{}/{}", Service::get_api(), COLLECTION_BASE_PATH, self.parent());
        let url = Url::parse(&url_string).unwrap();

        // POST request
        let ref mut api_client = self.service.api_client();
        let req = api_client.post(url).body(self.basename());

        // Parse response
        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));


        Service::decode_to_result::<CollectionCreated>(res_json)
    }


    /// Delete a collection
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// # use algorithmia::collection::Collection;
    /// let service = Service::new("111112222233333444445555566");
    /// let my_dir = service.collection("my_user/my_dir");
    /// match my_dir.delete() {
    ///   Ok(_) => println!("Successfully deleted collection"),
    ///   Err(e) => println!("ERROR deleting collection: {:?}", e),
    /// };
    /// ```
    pub fn delete(&'a self) -> CollectionDeletedResult {
        // DELETE request
        let ref mut api_client = self.service.api_client();
        let req = api_client.delete(self.to_url());

        // Parse response
        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Service::decode_to_result::<CollectionDeleted>(res_json)
    }


    /// Upload a file to an existing collection
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// # use algorithmia::collection::Collection;
    /// # use std::fs::File;
    /// let service = Service::new("111112222233333444445555566");
    /// let my_dir = service.collection("my_user/my_dir");
    ///
    /// match my_dir.upload_file("/path/to/file") {
    ///   Ok(response) => println!("Successfully uploaded to: {}", response.result),
    ///   Err(e) => println!("ERROR uploading file: {:?}", e),
    /// };
    /// ```
    pub fn upload_file<P: AsRef<Path>>(&'a self, file_path: P) -> CollectionFileAddedResult {
        // FIXME: A whole lot of unwrap going on here...
        let path_ref = file_path.as_ref();
        let url_string = format!("{}/{}",
            self.to_url(),
            path_ref.file_name().unwrap().to_str().unwrap()
        );
        let url = Url::parse(&url_string).unwrap();

        let mut file = File::open(path_ref).unwrap();
        let ref mut api_client = self.service.api_client();
        let req = api_client.post(url).body(&mut file);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Service::decode_to_result::<CollectionFileAdded>(res_json)
    }

    /// Write a file (raw bytes) directly to a data collection
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// # use algorithmia::collection::Collection;
    /// let service = Service::new("111112222233333444445555566");
    /// let my_dir = service.collection("my_user/my_dir");
    ///
    /// match my_dir.write_file("some_filename", "file_contents".as_bytes()) {
    ///   Ok(response) => println!("Successfully uploaded to: {}", response.result),
    ///   Err(e) => println!("ERROR uploading file: {:?}", e),
    /// };
    /// ```
    pub fn write_file(&'a self, filename: &str, input_data: &[u8]) -> CollectionFileAddedResult {
        let url_string = format!("{}/{}", self.to_url(), filename);
        let url = Url::parse(&url_string).unwrap();

        let ref mut api_client = self.service.api_client();
        let req = api_client.post(url).body(input_data);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Service::decode_to_result::<CollectionFileAdded>(res_json)
    }

    /// Delete a file from a data collection
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// # use algorithmia::collection::Collection;
    /// let service = Service::new("111112222233333444445555566");
    /// let my_dir = service.collection("my_user/my_dir");
    ///
    /// match my_dir.delete_file("some_filename") {
    ///   Ok(_) => println!("Successfully deleted file"),
    ///   Err(e) => println!("ERROR deleting file: {:?}", e),
    /// };
    /// ```
    pub fn delete_file(&'a self, filename: &str) -> CollectionFileDeletedResult {
        let url_string = format!("{}/{}", self.to_url(), filename);
        let url = Url::parse(&url_string).unwrap();

        let ref mut api_client = self.service.api_client();
        let req = api_client.delete(url);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Service::decode_to_result::<CollectionFileDeleted>(res_json)
    }

}

#[test]
fn test_to_url() {
    let collection = Collection { path: "anowell/foo", service: Service::new("")};
    assert_eq!(collection.to_url().serialize(), format!("{}/data/anowell/foo", Service::get_api()));
}

#[test]
fn test_parent() {
    let collection = Collection { path: "anowell/foo", service: Service::new("")};
    assert_eq!(collection.parent(), "anowell");
}