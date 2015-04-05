//! Algorithmia client library
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Service;
//! use algorithmia::algorithm::{Algorithm, AlgorithmOutput, Version};
//!
//! // Initialize with an API key
//! let algo_service = Service::new("111112222233333444445555566");
//! let factor = Algorithm::new("kenny", "Factor", Version::Revision(0,1,0));
//! let factor_service = algo_service.algorithm(&factor);
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = "19635".to_string();
//! let output: AlgorithmOutput<Vec<i64>> = factor_service.exec(&input).unwrap();
//! println!("Completed in {} seconds with result: {:?}", output.duration, output.result);
//! ```


#![doc(html_logo_url = "https://algorithmia.com/assets/images/apple-touch-icon.png")]

#![feature(slice_patterns)]
#![feature(file_path)]
extern crate hyper;
extern crate mime;
extern crate rustc_serialize;

pub mod algorithm;
pub mod collection;

use algorithm::{Algorithm, AlgorithmService};
use collection::{Collection, CollectionService};

use hyper::{Client, Url};
use hyper::client::RequestBuilder;
use hyper::header::{Accept, Authorization, ContentType, UserAgent, qitem};
use hyper::net::HttpConnector;
use mime::{Mime, TopLevel, SubLevel};
use rustc_serialize::{json, Decodable};
use self::AlgorithmiaError::*;
use std::io;

pub static API_BASE_URL: &'static str = "https://api.algorithmia.com";

/// The top-level struct for instantiating Algorithmia service endpoints
pub struct Service{
    api_key: String,
}

/// Internal ApiClient to manage connection and requests: wraps `hyper` client
pub struct ApiClient<'c>{
    api_key: String,
    client: Client<HttpConnector<'c>>,
    user_agent: String,
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

    /// Instantiate a new hyper client - used internally by instantiating new api_client for every request
    pub fn api_client(&self) -> ApiClient<'c> {
        ApiClient::new(self.api_key.clone())
    }

    /// Instantiate an `AlgorithmService` from this `Service`
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Service;
    /// use algorithmia::algorithm::{Algorithm, Version};
    /// let service = Service::new("111112222233333444445555566");
    /// let factor = Algorithm::new("anowell", "Dijkstra", Version::Latest);
    /// let factor_service = service.algorithm(&factor);
    /// ```
    pub fn algorithm(self, algorithm: &'a Algorithm<'a>) -> AlgorithmService<'a> {
        AlgorithmService {
            service: self,
            algorithm: algorithm,
        }
    }

    /// Instantiate a `CollectionService` from this `Service`
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Service;
    /// use algorithmia::collection::Collection;
    /// let service = Service::new("111112222233333444445555566");
    /// let rustfoo = Collection::new("anowell", "rustfoo");
    /// let rustfoo_service = service.collection(&rustfoo);
    /// ```
    pub fn collection(self, collection: &'a Collection<'a>) -> CollectionService<'a> {
        CollectionService {
            service: self,
            collection: collection,
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
    /// Instantiate an ApiClient - creates a new `hyper` client
    pub fn new(api_key: String) -> ApiClient<'c> {
        ApiClient {
            api_key: api_key,
            client: Client::new(),
            user_agent: format!("rust/{} algorithmia.rs/{}", option_env!("CFG_RELEASE").unwrap_or("unknown"), option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")),
        }
    }

    /// Helper to make Algorithmia GET requests with the API key
    pub fn get(&mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector> {
        self.client.get(url)
            .header(UserAgent(self.user_agent.clone()))
            .header(Authorization(self.api_key.clone()))
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn post(&mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector> {
        self.client.post(url)
            .header(UserAgent(self.user_agent.clone()))
            .header(Authorization(self.api_key.clone()))
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn delete(&mut self, url: Url) -> RequestBuilder<'c, Url, HttpConnector> {
        self.client.delete(url)
            .header(UserAgent(self.user_agent.clone()))
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
/// Allow cloning a service in order to reuse the API key for multiple connections
impl std::clone::Clone for Service {
    fn clone(&self) -> Service {
        Service {
            api_key: self.api_key.clone(),
        }
    }
}

impl From<io::Error> for AlgorithmiaError {
    fn from(err: io::Error) -> AlgorithmiaError {
        IoError(err)
    }
}

impl From<hyper::HttpError> for AlgorithmiaError {
    fn from(err: hyper::HttpError) -> AlgorithmiaError {
        HttpError(err)
    }
}

impl From<json::DecoderError> for AlgorithmiaError {
    fn from(err: json::DecoderError) -> AlgorithmiaError {
        DecoderError(err)
    }
}

impl From<json::EncoderError> for AlgorithmiaError {
    fn from(err: json::EncoderError) -> AlgorithmiaError {
        EncoderError(err)
    }
}
