//! Internal client
//!
//! Do not use directly - use the [`Algorithmia`](../struct.Algorithmia.html) struct instead
pub use hyper::client::response::Response;
use hyper::{Client, Url};
use hyper::client::RequestBuilder;
use hyper::header::{Authorization, UserAgent};
use hyper::method::Method;

use std::sync::Arc;
use std::clone;

pub use hyper::client::Body;

/// Represent the different ways to auth with the API
#[derive(Clone)]
pub enum ApiAuth {
    SimpleAuth(String),
    NoAuth
}

/// Internal HttpClient to build requests: wraps `hyper` client
pub struct HttpClient{
    pub base_url: String,
    api_auth: ApiAuth,
    hyper_client: Arc<Client>,
    user_agent: String,
}

impl HttpClient {
    /// Instantiate an HttpClient - creates a new `hyper` client
    pub fn new(api_auth: ApiAuth, base_url: String) -> HttpClient {
        HttpClient {
            api_auth: api_auth,
            base_url: base_url,
            hyper_client: Arc::new(Client::new()),
            user_agent: format!("algorithmia-rust/{} (Rust {}", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"), option_env!("CFG_RELEASE").unwrap_or("unknown")),
        }
    }

    /// Helper to make Algorithmia GET requests with the API key
    pub fn get(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::Get, url)
    }

    /// Helper to make Algorithmia GET requests with the API key
    pub fn head(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::Head, url)
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn post(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::Post, url)
    }

    /// Helper to make Algorithmia PUT requests with the API key
    pub fn put(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::Put, url)
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn delete(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::Delete, url)
    }


    fn build_request(&self, verb: Method, url: Url) -> RequestBuilder {
        let mut req = self.hyper_client.request(verb, url);

        // TODO: Support Secure Auth
        req = req.header(UserAgent(self.user_agent.clone()));
        if let ApiAuth::SimpleAuth(ref api_key) = self.api_auth {
           req = req.header(Authorization(format!("Simple {}", api_key)))
        }
        req
    }
}

/// Allow cloning in order to reuse http client (and API key) for multiple connections
impl clone::Clone for HttpClient {
    fn clone(&self) -> HttpClient {
        HttpClient {
            api_auth: self.api_auth.clone(),
            base_url: self.base_url.clone(),
            hyper_client: self.hyper_client.clone(),
            user_agent: self.user_agent.clone(),
        }
    }
}

impl <'a> From<&'a str> for ApiAuth {
    fn from(api_key: &'a str) -> Self {
        match api_key.len() {
            0 => ApiAuth::NoAuth,
            _ => ApiAuth::SimpleAuth(api_key.into()),
        }
    }
}

impl From<()> for ApiAuth {
    fn from(_: ()) -> Self {
        ApiAuth::NoAuth
    }
}