#![feature(io)]

extern crate hyper;
extern crate mime;
extern crate "rustc-serialize" as rustc_serialize;

pub mod algorithm;
pub use algorithm::{AlgorithmService,Algorithm,AlgorithmOutput};

use hyper::{Client, Url};
use hyper::client::RequestBuilder;
use hyper::header::{Authorization, ContentType};
use hyper::net::HttpConnector;
use mime::{Mime, TopLevel, SubLevel};
use rustc_serialize::{json};
use self::AlgorithmiaError::*;


pub struct Service<'c> {
    api_key: String,
    client: Client<HttpConnector<'c>>,
}

#[derive(Debug)]
pub enum AlgorithmiaError {
    HttpError(hyper::HttpError),
    DecoderError(json::DecoderError),
    EncoderError(json::EncoderError),
}

impl<'c> Service<'c> {
    pub fn new(api_key: &str) -> Service {
        Service {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    // Helper to inject API key and set MIME type
    pub fn post(&'c mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector<'c>> {
        self.client.post(url)
            .header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .header(Authorization(self.api_key.clone()))
    }

    pub fn algorithm(self, user: &'c str, repo: &'c str) -> AlgorithmService<'c> {
        AlgorithmService {
            service: self,
            algorithm: Algorithm { user: user, repo: repo }
        }
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
