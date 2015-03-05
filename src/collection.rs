extern crate hyper;

use ::{Service, AlgorithmiaError, ApiErrorResponse};
use hyper::Url;
use rustc_serialize::{json, Decoder};
use std::io::Read;
use std::fs::File;

static COLLECTION_BASE_URI: &'static str = "https://api.algorithmia.com/data";

pub struct Collection<'a> {
    pub user: &'a str,
    pub name: &'a str,
}

pub type CollectionCreatedResult = Result<CollectionCreated, AlgorithmiaError>;
pub type CollectionFileAddedResult = Result<CollectionFileAdded, AlgorithmiaError>;

#[derive(RustcDecodable, Debug)]
pub struct CollectionCreated {
    pub stream_id: u32,
    pub object_id: String,
    pub stream_name: String,
    pub username: String,
    pub acl: String,
}

#[derive(RustcDecodable, Debug)]
pub struct CollectionFileAdded {
    pub file_added: String
}

pub struct CollectionService<'a> {
    pub service: Service<'a>,
    pub collection: Collection<'a>,
}

impl<'a> Collection<'a> {
    fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}", COLLECTION_BASE_URI, self.user, self.name);
        Url::parse(&*url_string).unwrap()
    }
}

impl<'c> CollectionService<'c> {
    pub fn new(api_key: &'c str, user: &'c str, name: &'c str) -> CollectionService<'c> {
        CollectionService {
            service: Service::new(api_key),
            collection: Collection{ user: user, name: name }
        }
    }

    pub fn create(&'c mut self) -> CollectionCreatedResult {
        // Construct URL
        let url_string = format!("{}/{}", COLLECTION_BASE_URI, self.collection.user);
        let url = Url::parse(&*url_string).unwrap();

        // POST request
        let ref mut service = self.service;
        let req = service.post(url).body(self.collection.name);

        // Parse response
        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        match json::decode::<CollectionCreated>(&*res_json) {
            Ok(result) => Ok(result),
            Err(why) => match json::decode::<ApiErrorResponse>(&*res_json) {
                Ok(api_error) => Err(AlgorithmiaError::ApiError(api_error.error)),
                Err(_) => Err(AlgorithmiaError::DecoderErrorWithContext(why, res_json)),
            }
        }
    }

    pub fn upload_file(&'c mut self, file: &mut File) -> CollectionFileAddedResult {
        let url_string = format!("{}/{}",
            self.collection.to_url(),
            file.path().unwrap().file_name().unwrap().to_str().unwrap()
        );
        let url = Url::parse(&*url_string).unwrap();

        let ref mut service = self.service;
        let req = service.post(url).body(file);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));
        Ok(try!(json::decode(&*res_json)))
    }


    pub fn write_file(&'c mut self, filename: &str, input_data: &[u8]) -> CollectionFileAddedResult {
        let url_string = format!("{}/{}", self.collection.to_url(), filename);
        let url = Url::parse(&*url_string).unwrap();

        let ref mut service = self.service;
        let req = service.post(url).body(&*input_data);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));
        Ok(try!(json::decode(&*res_json)))
    }
}

#[test]
fn test_to_url() {
    let collection = Collection{ user: "anowell", name: "foo" };
    assert_eq!(collection.to_url().serialize(), "https://api.algorithmia.com/data/anowell/foo")
}
