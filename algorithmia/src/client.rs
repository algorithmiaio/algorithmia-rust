//! Internal client
//!
//! Do not use directly - use the [`Algorithmia`](../struct.Algorithmia.html) struct instead
use reqwest::{Client, Method, RequestBuilder, Url, IntoUrl};
use headers_ext::{HeaderMapExt, Authorization, UserAgent, Credentials};
use headers_core::{HeaderValue};
use http::header::HeaderMap;
use std::sync::Arc;
use std::str::FromStr;

pub use reqwest::Body;
use error::{ErrorKind, Result, ResultExt};

struct Simple(HeaderValue);
impl Credentials for Simple {
    const SCHEME: &'static str = "Simple";
    fn decode(value: &HeaderValue) -> Option<Self> {
        debug_assert!(
            value.as_bytes().starts_with(b"Simple "),
            "HeaderValue to decode should start with \"Simple ..\", received = {:?}",
            value,
        );

        Some(Simple(value.clone()))
    }
    fn encode(&self) -> HeaderValue {
        self.0.clone()
    }
}

impl Simple {
    /// Try to create a `Simple` authorization header.
    pub fn new(token: &str) -> Result<Self>  {
        HeaderValue::from_str(&format!("Simple {}", token))
            .map(Simple)
            .map_err(|_| ErrorKind::InvalidApiKey.into())
    }
}
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
    pub base_url: Url,
    api_auth: ApiAuth,
    inner_client: Arc<Client>,
    user_agent: String,
}

impl HttpClient {
    /// Instantiate an `HttpClient` - creates a new `reqwest` client
    pub fn new<U: IntoUrl>(api_auth: ApiAuth, base_url: U) -> Result<HttpClient> {
        Ok(HttpClient {
            api_auth: api_auth,
            base_url: base_url.into_url().chain_err(|| ErrorKind::InvalidBaseUrl)?,
            inner_client: Arc::new(Client::new()),
            user_agent: format!(
                "algorithmia-rust/{} (Rust {}",
                option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
                ::version::RUSTC_VERSION
            ),
        })
    }

    /// Helper to make Algorithmia GET requests with the API key
    pub fn get(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::GET, url)
    }

    /// Helper to make Algorithmia GET requests with the API key
    pub fn head(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::HEAD, url)
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn post(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::POST, url)
    }

    /// Helper to make Algorithmia PUT requests with the API key
    pub fn put(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::PUT, url)
    }

    /// Helper to make Algorithmia POST requests with the API key
    pub fn delete(&self, url: Url) -> RequestBuilder {
        self.build_request(Method::DELETE, url)
    }


    fn build_request(&self, verb: Method, url: Url) -> RequestBuilder {
        let mut headers = HeaderMap::new();
        headers.typed_insert(UserAgent::from_str(&self.user_agent).expect("User Agent not valid ASCII"));
        if let ApiAuth::ApiKey(ref api_key) = self.api_auth {
            headers.typed_insert(Authorization(Simple::new(api_key).expect("API Key not valid ASCII")));
        }

        self.inner_client.request(verb, url.clone()).headers(headers)
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
