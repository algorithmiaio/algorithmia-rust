extern crate hyper;
extern crate mime;
extern crate "rustc-serialize" as rustc_serialize;

use hyper::Url;
use hyper::header::{Authorization, ContentType};
use hyper::net::HttpConnector;
use mime::{Mime, TopLevel, SubLevel};
use rustc_serialize::{json, Decoder, Decodable, Encodable};
use self::AlgorithmiaError::{HttpError, DecoderError};

pub struct Algorithm<'a> {
    user: &'a str,
    repo: &'a str,
}

pub struct Client<'c, T> {
    api_key: String,
    hyper_client: hyper::Client<HttpConnector<'c>>,
    service: T
}

#[derive(RustcDecodable, Show)]
pub struct AlgorithmOutput<T> {
    pub duration: f32,
    pub result: T,
}

#[derive(Show)]
pub enum AlgorithmiaError {
    HttpError(hyper::HttpError),
    DecoderError(json::DecoderError),
}

pub type AlgorithmResult<T> = Result<AlgorithmOutput<T>, AlgorithmiaError>;
pub type AlgorithmJsonResult = Result<String, hyper::HttpError>;

impl<'a> Algorithm<'a> {
    fn to_url(&self) -> Url {
        let url_string = format!("https://api.algorithmia.com/api/{}/{}", self.user, self.repo);
        Url::parse(&*url_string).unwrap()
    }
}

impl<'c> Client<'c, ()> {
    pub fn new(api_key: &str) -> Client<()> {
        Client {
            api_key: api_key.to_string(),
            hyper_client: hyper::Client::new(),
            service: ()
        }
    }

    pub fn algorithm(self, user: &'c str, repo: &'c str) -> Client<'c, Algorithm<'c>> {
        Client {
            api_key: self.api_key,
            hyper_client: self.hyper_client,
            service: Algorithm{ user: user, repo: repo }
        }
    }

}

impl<'c> Client<'c, Algorithm<'c>> {
    pub fn query<'a, D, E>(self, input_data: &E) -> AlgorithmResult<D>
            where D: Decodable,
                  E: Encodable {
        let raw_input = json::encode(input_data);
        let json_output = try!(self.query_raw(&*raw_input));
        Ok(try!(json::decode(&*json_output)))
    }

    pub fn query_raw(self, input_data: &str) -> AlgorithmJsonResult {
        let mut client = self.hyper_client;
        let req = client.post(self.service.to_url())
            .header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .header(Authorization(self.api_key))
            .body(input_data);

        let mut res = try!(req.send());
        Ok(try!(res.read_to_string()))
    }

}

impl std::error::FromError<hyper::HttpError> for AlgorithmiaError {
    fn from_error(err: hyper::HttpError) -> AlgorithmiaError {
        HttpError(err)
    }
}

impl std::error::FromError<json::DecoderError> for AlgorithmiaError {
    fn from_error(err: json::DecoderError) -> AlgorithmiaError {
        DecoderError(err)
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
