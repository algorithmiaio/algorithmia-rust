extern crate hyper;

use ::{Service, AlgorithmiaError, ApiErrorResponse, API_BASE_URL};
use hyper::Url;
use rustc_serialize::{json, Decoder};
use std::io::Read;
use std::fs::File;

static COLLECTION_BASE_PATH: &'static str = "data";

pub struct Collection<'a> {
    pub user: &'a str,
    pub name: &'a str,
}

pub type CollectionShowResult = Result<CollectionShow, AlgorithmiaError>;
pub type CollectionCreatedResult = Result<CollectionCreated, AlgorithmiaError>;
pub type CollectionFileAddedResult = Result<CollectionFileAdded, AlgorithmiaError>;


#[derive(RustcDecodable, Debug)]
pub struct CollectionAcl {
    pub read_w: bool,
    pub read_g: bool,
    pub read_u: bool,
    pub read_a: bool,
}

#[derive(RustcDecodable, Debug)]
pub struct CollectionCreated {
    pub collection_id: u32,
    pub object_id: String,
    pub collection_name: String,
    pub username: String,
    pub acl: CollectionAcl,
}

#[derive(RustcDecodable, Debug)]
pub struct CollectionShow {
    pub username: String,
    pub collection_name: String,
    pub files: Vec<String>,
}

#[derive(RustcDecodable, Debug)]
pub struct CollectionFileAdded {
    pub result: String
}

pub struct CollectionService<'a> {
    pub service: Service,
    pub collection: Collection<'a>,
}

impl<'a> Collection<'a> {
    fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}/{}", API_BASE_URL, COLLECTION_BASE_PATH, self.user, self.name);
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

    pub fn show(&'c mut self) -> CollectionShowResult {

        let ref mut api_client = self.service.api_client();
        let req = api_client.get(self.collection.to_url());

        // Parse response
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

    pub fn create(&'c mut self) -> CollectionCreatedResult {
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

    pub fn upload_file(&'c mut self, file: &mut File) -> CollectionFileAddedResult {
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


    pub fn write_file(&'c mut self, filename: &str, input_data: &[u8]) -> CollectionFileAddedResult {
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
