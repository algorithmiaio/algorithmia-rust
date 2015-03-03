extern crate hyper;

use ::{Service, AlgorithmiaError};
use hyper::Url;
use rustc_serialize::{json, Decoder, Decodable, Encodable};
use std::io::Read;

pub struct Algorithm<'a> {
    pub user: &'a str,
    pub repo: &'a str,
}

pub type AlgorithmResult<T> = Result<AlgorithmOutput<T>, AlgorithmiaError>;
pub type AlgorithmJsonResult = Result<String, hyper::HttpError>;

#[derive(RustcDecodable, Debug)]
pub struct AlgorithmOutput<T> {
    pub duration: f32,
    pub result: T,
}

pub struct AlgorithmService<'a> {
    pub service: Service<'a>,
    pub algorithm: Algorithm<'a>,
}

impl<'a> Algorithm<'a> {
    fn to_url(&self) -> Url {
        let url_string = format!("https://api.algorithmia.com/api/{}/{}", self.user, self.repo);
        Url::parse(&*url_string).unwrap()
    }
}

impl<'c> AlgorithmService<'c> {
    pub fn new(api_key: &'c str, user: &'c str, repo: &'c str) -> AlgorithmService<'c> {
        AlgorithmService {
            service: Service::new(api_key),
            algorithm: Algorithm{ user: user, repo: repo }
        }
    }

    pub fn query<'a, D, E>(&'c mut self, input_data: &E) -> AlgorithmResult<D>
            where D: Decodable,
                  E: Encodable {
        let raw_input = try!(json::encode(input_data));
        let json_output = try!(self.query_raw(&*raw_input));
        Ok(try!(json::decode(&*json_output)))
    }

    pub fn query_raw(&'c mut self, input_data: &str) -> AlgorithmJsonResult {
        let ref mut service = self.service;
        let req = service.post(self.algorithm.to_url())
            .body(input_data);

        let mut res = try!(req.send());
        let mut res_string = String::new();
        try!(res.read_to_string(&mut res_string));
        Ok(res_string)
    }

}

#[test]
fn test_to_url() {
    let algorithm = Algorithm{ user: "kenny", repo: "Factor" };
    assert_eq!(algorithm.to_url().serialize(), "https://api.algorithmia.com/api/kenny/Factor")
}

#[test]
fn test_json_decoding() {
    let json_output = "{\"duration\":0.46739511,\"result\":[5,41]}";
    let expected = AlgorithmOutput{ duration: 0.46739511f32, result: [5, 41] };
    let decoded: AlgorithmOutput<Vec<i32>> = json::decode(json_output).unwrap();
    assert_eq!(expected.duration, decoded.duration);
    assert_eq!(expected.result, decoded.result);
}
