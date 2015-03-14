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

pub struct Service{
    api_key: String,
    // client: Client<HttpConnector<'c>>,
}

pub struct ApiClient<'c>{
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

impl<'c> ApiClient<'c> {
    pub fn new(api_key: &str) -> ApiClient {
        ApiClient {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    // Helper to inject API key
    pub fn get(&mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector> {
        // let client = self.client ;
        self.client.get(url)
            .header(Authorization(self.api_key.clone()))
    }

    // Helper to inject API key
    pub fn post(&mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector> {
        // let client = self.client;
        self.client.post(url)
            .header(Authorization(self.api_key.clone()))
    }

    // Helper to add the MIME type
    pub fn post_json(&mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector> {
        self.post(url)
            .header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]))
    }
}


impl<'a, 'c> Service {
    pub fn new(api_key: &str) -> Service {
        Service {
            api_key: api_key.to_string(),
        }
    }

    pub fn api_client(&self) -> ApiClient<'c> {
        ApiClient {
            api_key: self.api_key.clone(),
            client: Client::new(),
        }
    }

    pub fn algorithm(self, user: &'a str, repo: &'a str) -> AlgorithmService<'a> {
        AlgorithmService {
            service: self,
            algorithm: Algorithm { user: user, repo: repo }
        }
    }

    pub fn collection(self, user: &'a str, name: &'a str) -> CollectionService<'a> {
        CollectionService {
            service: self,
            collection: Collection { user: user, name: name }
        }
    }

}


impl std::clone::Clone for Service {
    fn clone(&self) -> Service {
        Service {
            api_key: self.api_key.clone(),
            // client: Client::new(),
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
