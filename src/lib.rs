extern crate hyper;
extern crate mime;
extern crate "rustc-serialize" as rustc_serialize;

use hyper::{HttpError, Url};
use hyper::header::common::authorization::Authorization;
use hyper::header::common::content_type::ContentType;
use hyper::net::HttpConnector;
use mime::{Mime, TopLevel, SubLevel};
use rustc_serialize::{json, Decoder, Decodable, Encoder, Encodable};
use std::io::IoError;

pub struct Algorithm {
    user: String,
    repo: String,
}

pub struct Client {
    api_key: String,
    hyper_client: hyper::Client<HttpConnector>,
}

#[deriving(RustcDecodable, Show)]
pub struct Output<T> {
    pub duration: f32,
    pub result: T,
}

pub type AlgorithmResult<T> = Result<Output<T>, HttpError>;
pub type AlgorithmRawResult = Result<String, HttpError>;

impl Algorithm {
    pub fn new(user: &str, repo: &str) -> Algorithm {
        Algorithm { user: user.to_string(), repo: repo.to_string() }
    }

    fn to_url(&self) -> Url {
        let url_string = format!("https://api.algorithmia.com/api/{}/{}", self.user, self.repo);
        Url::parse(url_string.as_slice()).unwrap()
    }
}

impl Client {
    pub fn new(api_key: &str) -> Client {
        Client {
            api_key: api_key.to_string(),
            hyper_client: hyper::Client::new(),
        }
    }

    pub fn query<'a, D, E>(self, algorithm: Algorithm, input_data: &E) -> AlgorithmResult<D>
            where D: Decodable<json::Decoder, json::DecoderError>,
                  E: Encodable<json::Encoder<'a>, IoError> {
        let raw_input = json::encode(input_data);
        let raw = try!(self.query_raw(algorithm, raw_input.as_slice()));
        Ok(json::decode(raw.as_slice()).unwrap())
    }

    pub fn query_raw(self, algorithm: Algorithm, input_data: &str) -> AlgorithmRawResult {
        let mut client = self.hyper_client;
        let req = client.post(algorithm.to_url())
            .header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .header(Authorization(self.api_key))
            .body(input_data);

        let mut res = try!(req.send());
        Ok(try!(res.read_to_string()))
    }
}

#[test]
fn test_to_url() {
    let algorithm = Algorithm::new("kenny", "Factor");
    assert_eq!(algorithm.to_url().serialize(), "https://api.algorithmia.com/api/kenny/Factor")
}

#[test]
fn test_json_decoding() {
    let json_output = "{\"duration\":0.46739511,\"result\":[5,41]}";
    let expected = Output{ duration: 0.46739511f32, result: [5, 41] };
    let decoded: Output<Vec<int>> = json::decode(json_output).unwrap();
    assert_eq!(expected.duration, decoded.duration);
    assert_eq!(expected.result, decoded.result);
}
