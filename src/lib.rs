#![feature(io)]
#![feature(path)]

extern crate hyper;
extern crate mime;
extern crate "rustc-serialize" as rustc_serialize;

pub mod algorithm;
pub mod collection;

pub use algorithm::{AlgorithmService,Algorithm,AlgorithmOutput};
pub use collection::{CollectionService,Collection,CollectionCreated};

use hyper::{Client, Url};
use hyper::client::RequestBuilder;
use hyper::header::{Accept, Authorization, ContentType, qitem};
use hyper::net::HttpConnector;
use mime::{Mime, TopLevel, SubLevel};
use rustc_serialize::{json};
use self::AlgorithmiaError::*;
use std::io;

pub static API_BASE_URL: &'static str = "https://api.algorithmia.com";

pub struct Service<'c> {
    api_key: String,
    client: Client<HttpConnector<'c>>,
}

// pub type ApiError = String;

#[derive(Debug)]
pub enum AlgorithmiaError {
    ApiError(String),
    HttpError(hyper::HttpError),
    DecoderError(json::DecoderError),
    DecoderErrorWithContext(json::DecoderError, String),
    EncoderError(json::EncoderError),
    IoError(io::Error),
}

#[derive(RustcDecodable, Debug)]
pub struct ApiErrorResponse {
    pub error: String,
}

impl<'c> Service<'c> {
    pub fn new(api_key: &str) -> Service {
        Service {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    // Helper to inject API key
    pub fn post(&'c mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector<'c>> {
        self.client.post(url)
            .header(Authorization(self.api_key.clone()))
    }

    // Helper to add the MIME type
    pub fn post_json(&'c mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector<'c>> {
        self.post(url)
            .header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]))
    }


    pub fn algorithm(self, user: &'c str, repo: &'c str) -> AlgorithmService<'c> {
        AlgorithmService {
            service: self,
            algorithm: Algorithm { user: user, repo: repo }
        }
    }

    pub fn collection(self, user: &'c str, name: &'c str) -> CollectionService<'c> {
        CollectionService {
            service: self,
            collection: Collection { user: user, name: name }
        }
    }

}


impl std::error::FromError<io::Error> for AlgorithmiaError {
    fn from_error(err: io::Error) -> AlgorithmiaError {
        IoError(err)
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

impl std::error::FromError<json::EncoderError> for AlgorithmiaError {
    fn from_error(err: json::EncoderError) -> AlgorithmiaError {
        EncoderError(err)
    }
}
