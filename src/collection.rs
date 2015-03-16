//! Algorithm module for managing Algorithmia Data Collections
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Service;
//! use std::fs::File;
//!
//! let algo_service = Service::new("111112222233333444445555566");
//! let my_bucket = algo_service.collection("my_user", "my_bucket");
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
pub type CollectionFileAddedResult = Result<CollectionFileAdded, AlgorithmiaError>;


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

/// Service endpoint for managing Algorithmia data collections
pub struct CollectionService<'a> {
    pub service: Service,
    pub collection: Collection<'a>,
}

impl<'a> Collection<'a> {
    /// Get the API Endpoint URL for a particular collection
    fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}/{}", API_BASE_URL, COLLECTION_BASE_PATH, self.user, self.name);
        Url::parse(&*url_string).unwrap()
    }
}

impl<'c> CollectionService<'c> {
    /// Instantiate `CollectionService` directly - alternative to `Service::collection`
    ///
    /// # Examples
    /// ```
    /// # use algorithmia::collection::CollectionService;
    /// let my_bucket = CollectionService::new("111112222233333444445555566", "my_user", "my_bucket");
    /// ```
    pub fn new(api_key: &'c str, user: &'c str, name: &'c str) -> CollectionService<'c> {
        CollectionService {
            service: Service::new(api_key),
            collection: Collection{ user: user, name: name }
        }
    }

    /// Display collection details if it exists
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection("my_user", "my_bucket");
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
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection("my_user", "my_bucket");
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

    /// Upload a file to an existing collection
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Service;
    /// # use std::fs::File;
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection("my_user", "my_bucket");
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
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let my_bucket = algo_service.collection("my_user", "my_bucket");
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


}

#[test]
fn test_to_url() {
    let collection = Collection{ user: "anowell", name: "foo" };
    assert_eq!(collection.to_url().serialize(), format!("{}/data/anowell/foo", API_BASE_URL));
}
