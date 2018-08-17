//! Internal client
//!
//! Do not use directly - use the [`Algorithmia`](../struct.Algorithmia.html) struct instead
use reqwest::{Client, Method, RequestBuilder, Url, IntoUrl};
use reqwest::header::{Authorization, UserAgent};

use url::ParseError;
use std::sync::Arc;

pub use reqwest::Body;

/// Represent the different ways to auth with the API
#[derive(Clone)]
pub enum ApiAuth {
    /// Algorithmia API key to use for authentication
    ApiKey(String),
    /// Use unauthenticated request (common for on-platform algorithms)
    None,
}

/// Internal `HttpClient` to build requests: wraps `reqwest` client
#[derive(Clone)]
pub struct HttpClient {
    pub base_url: ::std::result::Result<Url, ParseError>,
    api_auth: ApiAuth,
    inner_client: Arc<Client>,
    user_agent: String,
}

impl HttpClient {
    /// Instantiate an `HttpClient` - creates a new `reqwest` client
    pub fn new<U: IntoUrl>(api_auth: ApiAuth, base_url: U) -> HttpClient {
        HttpClient {
            api_auth: api_auth,
            base_url: base_url.into_url(),
            inner_client: Arc::new(Client::new()),
            user_agent: format!(
                "algorithmia-rust/{} (Rust {}",
                option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
                ::version::RUSTC_VERSION
            ),
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
        let mut req = self.inner_client.request(verb, url.clone());

        req.header(UserAgent::new(self.user_agent.clone()));
        if let ApiAuth::ApiKey(ref api_key) = self.api_auth {
            req.header(Authorization(format!("Simple {}", api_key)));
        }
        req
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
