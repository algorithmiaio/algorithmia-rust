//! Algorithmia client library
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Service;
//! use algorithmia::algorithm::{Algorithm, AlgorithmOutput, Version};
//!
//! // Initialize with an API key
//! let service = Service::new("111112222233333444445555566");
//! let moving_avg = service.algorithm("timeseries", "SimpleMovingAverage", Version::Minor(0,1));
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = (vec![0,1,2,3,15,4,5,6,7], 3);
//! let output: AlgorithmOutput<Vec<f64>> = moving_avg.pipe(&input).unwrap();
//! println!("Completed in {} seconds with result: {:?}", output.duration, output.result);
//! ```

#![doc(html_logo_url = "https://algorithmia.com/assets/images/apple-touch-icon.png")]

extern crate hyper;
extern crate mime;
extern crate rustc_serialize;

pub mod algorithm;
pub mod collection;

use algorithm::{Algorithm, Version};
use collection::{Collection};

use hyper::{Client, Url};
use hyper::client::RequestBuilder;
use hyper::header::{Authorization, UserAgent};
use rustc_serialize::{json, Decodable};
use self::AlgorithmiaError::*;
use std::{io, env};

static DEFAULT_API_BASE_URL: &'static str = "https://api.algorithmia.com";

/// The top-level struct for instantiating Algorithmia service endpoints
pub struct Service{
    pub api_key: String,
}

/// Internal ApiClient to manage connection and requests: wraps `hyper` client
pub struct ApiClient{
    api_key: String,
    client: Client,
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

    pub fn get_api() -> String {
        // TODO: memoize
        match env::var("ALGORITHMIA_API") {
            Ok(url) => url,
            Err(_) => DEFAULT_API_BASE_URL.to_string(),
        }
    }

    /// Instantiate a new hyper client - used internally by instantiating new api_client for every request
    pub fn api_client(&self) -> ApiClient {
        ApiClient::new(self.api_key.to_string())
    }

    /// Instantiate an `AlgorithmService` from this `Service`
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Service;
    /// use algorithmia::algorithm::{Algorithm, Version};
    /// let service = Service::new("111112222233333444445555566");
    /// let factor = service.algorithm("anowell", "Dijkstra", Version::Latest);
    /// ```
    pub fn algorithm(self, user: &'a str, repo: &'a str, version: Version<'a>) -> Algorithm<'a> {
        Algorithm {
            service: self,
            user: user,
            repo: repo,
            version: version
        }
    }

    /// Instantiate an algorithm from the algorithm's URI
    ///
    /// # Examples
    /// ```
    /// use algorithmia::Service;
    /// use algorithmia::algorithm::{Algorithm, Version};
    /// let service = Service::new("111112222233333444445555566");
    /// let factor = service.algorithm_from_str("anowell/Dijkstra/0.1");
    /// ```
    pub fn algorithm_from_str(self, algo_uri: &'a str) -> Result<Algorithm<'a>, &'a str> {
        // TODO: test that this works for stripping algo:// prefix
        // let stripped = match algo_uri.rsplitn(2, "//").nth(0) {
        //     Some(path) => path,
        //     None => return Err("Invalid algorithm URI"),
        // };

        let parts: Vec<_> = algo_uri.split("/").collect();
        match parts.len() {
            3 => Ok(
                Algorithm {
                    service: self,
                    user: parts[0],
                    repo: parts[1],
                    version: Version::from_str(parts[2])
                }
            ),
            2 => Ok(
                Algorithm {
                    service: self,
                    user: parts[0],
                    repo: parts[1],
                    version: Version::Latest
                }
            ),
            _ => Err("Invalid algorithm URI")
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
    /// let rustfoo = service.collection("anowell/rustfoo");
    /// ```
    pub fn collection(self, path: &'a str) -> Collection<'a> {
        Collection {
            service: self,
            path: path,
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

impl ApiClient {
    /// Instantiate an ApiClient - creates a new `hyper` client
    pub fn new(api_key: String) -> ApiClient {
        ApiClient {
            api_key: api_key,
            client: Client::new(),
            user_agent: format!("rust/{} algorithmia.rs/{}", option_env!("CFG_RELEASE").unwrap_or("unknown"), option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")),
        }
    }

    /// Helper to make Algorithmia GET requests with the API key
    pub fn get(&mut self, url: Url) -> RequestBuilder<Url> {
        self.client.get(url)
            .header(UserAgent(self.user_agent.clone()))
            .header(Authorization(self.api_key.clone()))
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn post(&mut self, url: Url) -> RequestBuilder<Url> {
        self.client.post(url)
            .header(UserAgent(self.user_agent.clone()))
            .header(Authorization(self.api_key.clone()))
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn delete(&mut self, url: Url) -> RequestBuilder<Url> {
        self.client.delete(url)
            .header(UserAgent(self.user_agent.clone()))
            .header(Authorization(self.api_key.clone()))
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