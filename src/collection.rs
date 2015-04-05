//! Collection module for managing Algorithmia Data Collections
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Service;
//! use algorithmia::collection::Collection;
//! use std::fs::File;
//!
//! let service = Service::new("111112222233333444445555566");
//! let my_bucket = service.collection(Collection::new("my_user", "my_bucket"));
//!
//! my_bucket.create();
//! let mut my_file = File::open("/path/to/file").unwrap();
//! my_bucket.upload_file(&mut my_file);
//!
//! my_bucket.write_file("some_filename", "file_contents".as_bytes());
//! ```

extern crate hyper;

use ::{Service, AlgorithmiaError, ApiErrorResponse, API_BASE_URL};
use hyper::Url;
use rustc_serialize::{json, Decoder};
use std::io::Read;
use std::fs::File;

static COLLECTION_BASE_PATH: &'static str = "data";

/// Algorithmia data collection
pub struct Collection<'a> {
    pub user: &'a str,
    pub name: &'a str,
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

/// Service endpoint for managing Algorithmia data collections
pub struct CollectionService<'a> {
    pub service: Service,
    pub collection: Collection<'a>,
}

impl<'a> Collection<'a> {
    /// Initializes a regular Collection from the username and collection name
    ///
    /// # Examples
    /// ```
    /// # use algorithmia::collection::Collection;
    /// let collection = Collection::new("anowell", "foo");
    /// assert_eq!(collection.user, "anowell");
    /// assert_eq!(collection.name, "foo");
    /// ```
    pub fn new(user: &'a str, name: &'a str) -> Collection<'a> {
        Collection {
            user: user,
            name: name,
        }
    }

    /// Initializes a Collection from the data_uri
    ///
    /// # Examples
    /// ```
    /// # use algorithmia::collection::Collection;
    /// let collection = Collection::from_str("anowell/foo").ok().unwrap();
    /// assert_eq!(collection.user, "anowell");
    /// assert_eq!(collection.name, "foo");
    /// ```
    pub fn from_str(data_uri: &'a str) -> Result<Collection<'a>, &'a str> {
        // TODO: strip optional 'data://' prefix
        match &*data_uri.split("/").collect::<Vec<_>>() {
            [user, collection_name] => Ok(Collection{user: user, name: collection_name}),
            _ => Err("Invalid collection URI")
        }
    }

    /// Get the API Endpoint URL for a particular collection
    pub fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}/{}", API_BASE_URL, COLLECTION_BASE_PATH, self.user, self.name);
        Url::parse(&*url_string).unwrap()
    }
}

impl<'c> CollectionService<'c> {
    /// Display collection details if it exists
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// # use algorithmia::collection::Collection;
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection(Collection::new("my_user", "my_bucket"));
    /// match my_bucket.show() {
    ///   Ok(bucket) => println!("Files: {}", bucket.files.connect(", ")),
    ///   Err(e) => println!("ERROR: {:?}", e),
    /// };
    /// ```
    pub fn show(&'c self) -> CollectionShowResult {
        let ref mut api_client = self.service.api_client();
        let req = api_client.get(self.collection.to_url());

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        match json::decode::<CollectionShow>(&*res_json) {
            Ok(result) => Ok(result),
            Err(why) => match json::decode::<ApiErrorResponse>(&*res_json) {
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
    /// # use algorithmia::collection::Collection;
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection(Collection::new("my_user", "my_bucket"));
    /// match my_bucket.create() {
    ///   Ok(_) => println!("Successfully created collection"),
    ///   Err(e) => println!("ERROR creating collection: {:?}", e),
    /// };
    /// ```
    pub fn create(&'c self) -> CollectionCreatedResult {
        // Construct URL
        let url_string = format!("{}/{}/{}", API_BASE_URL, COLLECTION_BASE_PATH, self.collection.user);
        let url = Url::parse(&*url_string).unwrap();

        // POST request
        let ref mut api_client = self.service.api_client();
        let req = api_client.post(url).body(self.collection.name);

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
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection(Collection::new("my_user", "my_bucket"));
    /// match my_bucket.delete() {
    ///   Ok(_) => println!("Successfully deleted collection"),
    ///   Err(e) => println!("ERROR deleting collection: {:?}", e),
    /// };
    /// ```
    pub fn delete(&'c self) -> CollectionDeletedResult {
        // DELETE request
        let ref mut api_client = self.service.api_client();
        let req = api_client.delete(self.collection.to_url());

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
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection(Collection::new("my_user", "my_bucket"));
    ///
    /// let mut my_file = File::open("/path/to/file").unwrap();
    /// match my_bucket.upload_file(&mut my_file) {
    ///   Ok(response) => println!("Successfully uploaded to: {}", response.result),
    ///   Err(e) => println!("ERROR uploading file: {:?}", e),
    /// };
    /// ```
    pub fn upload_file(&'c self, file: &mut File) -> CollectionFileAddedResult {
        let url_string = format!("{}/{}",
            self.collection.to_url(),
            file.path().unwrap().file_name().unwrap().to_str().unwrap()
        );
        let url = Url::parse(&*url_string).unwrap();

        let ref mut api_client = self.service.api_client();
        let req = api_client.post(url).body(file);

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
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection(Collection::new("my_user", "my_bucket"));
    ///
    /// match my_bucket.write_file("some_filename", "file_contents".as_bytes()) {
    ///   Ok(response) => println!("Successfully uploaded to: {}", response.result),
    ///   Err(e) => println!("ERROR uploading file: {:?}", e),
    /// };
    /// ```
    pub fn write_file(&'c self, filename: &str, input_data: &[u8]) -> CollectionFileAddedResult {
        let url_string = format!("{}/{}", self.collection.to_url(), filename);
        let url = Url::parse(&*url_string).unwrap();

        let ref mut api_client = self.service.api_client();
        let req = api_client.post(url).body(&*input_data);

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
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection(Collection::new("my_user", "my_bucket"));
    ///
    /// match my_bucket.delete_file("some_filename") {
    ///   Ok(_) => println!("Successfully deleted file"),
    ///   Err(e) => println!("ERROR deleting file: {:?}", e),
    /// };
    /// ```
    pub fn delete_file(&'c self, filename: &str) -> CollectionFileDeletedResult {
        let url_string = format!("{}/{}", self.collection.to_url(), filename);
        let url = Url::parse(&*url_string).unwrap();

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
    let collection = Collection{ user: "anowell", name: "foo" };
    assert_eq!(collection.to_url().serialize(), format!("{}/data/anowell/foo", API_BASE_URL));
}
