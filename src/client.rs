//! Internal client
//!
//! Do not use directly - use the [`Algorithmia`](../struct.Algorithmia.html) struct instead
use hyper::{Client, Url};
use hyper::client::IntoUrl;
use hyper::client::RequestBuilder;
use hyper::header::{Authorization, UserAgent};
use hyper::method::Method;
use ::Error;

use url::ParseError;
use std::sync::Arc;

/// Represent the different ways to auth with the API
#[derive(Clone)]
pub enum ApiAuth {
    ApiKey(String),
    None,
}

/// Internal `HttpClient` to build requests: wraps `hyper` client
#[derive(Clone)]
pub struct HttpClient {
    pub base_url: Result<Url, ParseError>,
    api_auth: ApiAuth,
    hyper_client: Arc<Client>,
    user_agent: String,
}

impl HttpClient {
    /// Instantiate an `HttpClient` - creates a new `hyper` client
    pub fn new<U: IntoUrl>(api_auth: ApiAuth, base_url: U) -> HttpClient {
        HttpClient {
            api_auth: api_auth,
            base_url: base_url.into_url(),
            hyper_client: Arc::new(Client::new()),
            user_agent: format!("algorithmia-rust/{} (Rust {}",
                                option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
                                ::version::RUSTC_VERSION),
        }
    }

    /// Helper to make Algorithmia GET requests with the API key
    pub fn get(&self, path: &str) -> Result<RequestBuilder, Error> {
        self.build_request(Method::Get, path)
    }

    /// Helper to make Algorithmia GET requests with the API key
    pub fn head(&self, path: &str) -> Result<RequestBuilder, Error> {
        self.build_request(Method::Head, path)
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn post(&self, path: &str) -> Result<RequestBuilder, Error> {
        self.build_request(Method::Post, path)
    }

    /// Helper to make Algorithmia PUT requests with the API key
    pub fn put(&self, path: &str) -> Result<RequestBuilder, Error> {
        self.build_request(Method::Put, path)
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn delete(&self, path: &str) -> Result<RequestBuilder, Error> {
        self.build_request(Method::Delete, path)
    }


    fn build_request(&self, verb: Method, path: &str) -> Result<RequestBuilder, Error> {
        let base_url = match self.base_url {
            Ok(ref u) => u,
            Err(e) => return Err(e.into()),
        };

        let url = try!(base_url.join(path));
        let mut req = self.hyper_client.request(verb, url);

        req = req.header(UserAgent(self.user_agent.clone()));
        if let ApiAuth::ApiKey(ref api_key) = self.api_auth {
            req = req.header(Authorization(format!("Simple {}", api_key)))
        }
        Ok(req)
    }
}

impl<'a> From<&'a str> for ApiAuth {
    fn from(api_key: &'a str) -> Self {
        match api_key.len() {
            0 => ApiAuth::None,
            _ => ApiAuth::ApiKey(api_key.into()),
        }
    }
}

impl From<String> for ApiAuth {
    fn from(api_key: String) -> Self {
        match api_key.len() {
            0 => ApiAuth::None,
            _ => ApiAuth::ApiKey(api_key),
        }
    }
}

impl From<()> for ApiAuth {
    fn from(_: ()) -> Self {
        ApiAuth::None
    }
}
