//! Algorithmia client library
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Service;
//! use algorithmia::algorithm::AlgorithmOutput;
//!
//! // Initialize with an API key
//! let algo_service = Service::new("111112222233333444445555566");
//! let mut factor = algo_service.algorithm("kenny", "Factor");
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = "19635".to_string();
//! let output: AlgorithmOutput<Vec<i64>> = factor.exec(&input).unwrap();
//! println!("Completed in {} seconds with result: {:?}", output.duration, output.result);
//! ```

extern crate hyper;
extern crate mime;
extern crate "rustc-serialize" as rustc_serialize;

pub mod algorithm;
pub mod collection;

use algorithm::{AlgorithmService,Algorithm,AlgorithmOutput};
use collection::{CollectionService,Collection,CollectionCreated};

use hyper::{Client, Url};
use hyper::client::RequestBuilder;
use hyper::header::{Accept, Authorization, ContentType, qitem};
use hyper::net::HttpConnector;
use mime::{Mime, TopLevel, SubLevel};
use rustc_serialize::{json, Decodable};
use self::AlgorithmiaError::*;
use std::io;

pub static API_BASE_URL: &'static str = "https://api.algorithmia.com";

/// The top-level struct for instantiating Algorithmia service endpoints
pub struct Service{
    pub api_key: String,
}

/// Internal ApiClient - generally instantiated by `Service::api_client`
pub struct ApiClient<'c>{
    api_key: String,
    client: Client<HttpConnector<'c>>,
}

/// Errors that may be returned by this library
#[derive(Debug)]
pub enum AlgorithmiaError {
    /// Errors returned by the Algorithmia API
    ApiError(String), //TODO: add the optional stacktrace or use ApiErrorResponse directly
    /// HTTP errors encountered by the hyper client
    HttpError(hyper::HttpError),
    /// Errors decoding response json
    DecoderError(json::DecoderError),
    /// Errors decoding response json with additional debugging context
    DecoderErrorWithContext(json::DecoderError, String),
    /// Errors encoding the request
    EncoderError(json::EncoderError),
    /// General IO errors
    IoError(io::Error),
}

/// Struct for decoding Algorithmia API error responses
#[derive(RustcDecodable, Debug)]
pub struct ApiErrorResponse {
    pub error: String,
    pub stacktrace: Option<String>,
}


impl<'a, 'c> Service {
    /// Instantiate a new Service
    pub fn new(api_key: &str) -> Service {
        Service {
            api_key: api_key.to_string(),
        }
    }

    /// Instantiate a new hyper client for each request through this method
    pub fn api_client(&self) -> ApiClient<'c> {
        ApiClient {
            api_key: self.api_key.clone(),
            client: Client::new(),
        }
    }

    /// Instantiate an `AlgorithmService` from this `Service`
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Service;
    /// let service = Service::new("111112222233333444445555566");
    /// let factor = service.algorithm("anowell", "Dijkstra");
    /// ```
    pub fn algorithm(self, user: &'a str, repo: &'a str) -> AlgorithmService<'a> {
        AlgorithmService {
            service: self,
            algorithm: Algorithm { user: user, repo: repo }
        }
    }

    /// Instantiate a `CollectionService` from this `Service`
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Service;
    /// let service = Service::new("111112222233333444445555566");
    /// let factor = service.algorithm("anowell", "rustfoo");
    /// ```
    pub fn collection(self, user: &'a str, name: &'a str) -> CollectionService<'a> {
        CollectionService {
            service: self,
            collection: Collection { user: user, name: name }
        }
    }

    /// Helper to standardize decoding to a specific Algorithmia Result type
    pub fn decode_to_result<T: Decodable>(res_json: String) -> Result<T, AlgorithmiaError> {
        match json::decode::<T>(&*res_json) {
            Ok(result) => Ok(result),
            Err(why) => match json::decode::<ApiErrorResponse>(&*res_json) {
                Ok(api_error) => Err(AlgorithmiaError::ApiError(api_error.error)),
                Err(_) => Err(AlgorithmiaError::DecoderErrorWithContext(why, res_json)),
            }
        }
    }

}

impl<'c> ApiClient<'c> {
    pub fn new(api_key: &str) -> ApiClient {
        ApiClient {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    /// Helper to make Algorithmia GET requests with the API key
    pub fn get(&mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector> {
        // let client = self.client ;
        self.client.get(url)
            .header(Authorization(self.api_key.clone()))
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn post(&mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector> {
        // let client = self.client;
        self.client.post(url)
            .header(Authorization(self.api_key.clone()))
    }

    /// Helper to POST JSON to Algorithmia with the correct Mime types
    pub fn post_json(&mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector> {
        self.post(url)
            .header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .header(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]))
    }
}


/*
* Trait implementations
*/
/// Allowing cloning a service in order to reuse the API key for multiple connections
impl std::clone::Clone for Service {
    fn clone(&self) -> Service {
        Service {
            api_key: self.api_key.clone(),
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
