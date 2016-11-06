//! Algorithmia client library
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Algorithmia;
//! use algorithmia::algo::{Algorithm, Version};
//!
//! // Initialize with an API key
//! let client = Algorithmia::client("111112222233333444445555566");
//! let moving_avg = client.algo(("timeseries/SimpleMovingAverage", "0.1"));
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = (vec![0,1,2,3,15,4,5,6,7], 3);
//! let result: Vec<f64> = moving_avg.pipe(&input).unwrap().decode().unwrap();
//! println!("Completed with result: {:?}", result);
//! ```

#![doc(html_logo_url = "https://algorithmia.com/assets/images/logos/png/bintreePurple.png")]

#![cfg_attr(feature="with-serde", feature(proc_macro))]

#[cfg(feature="with-serde")] #[macro_use] extern crate serde_derive;
#[macro_use] extern crate hyper;
#[macro_use] extern crate quick_error;

#[cfg(feature="with-serde")] extern crate serde;
#[cfg(feature="with-serde")] extern crate serde_json;
#[cfg(feature="with-rustc-serialize")] extern crate rustc_serialize;
extern crate base64;
extern crate chrono;
extern crate url;

use algo::{Algorithm, AlgoRef};
use data::{DataDir, DataFile, DataObject, HasDataPath};
use client::HttpClient;

use std::rc::Rc;

pub mod algo;
pub mod data;
pub mod error;
mod client;
pub use error::Error;
pub use hyper::{mime, Url};
use hyper::client::IntoUrl;
pub use client::ApiAuth;
pub use hyper::client::Body;

#[cfg(feature="with-serde")]
#[path = "json-serde.rs"]
mod json;

#[cfg(feature="with-rustc-serialize")]
#[path = "json-rustc-serialize.rs"]
mod json;

/// Reexports of the most common types and traits
pub mod prelude {
    pub use ::Algorithmia;
    pub use ::client::ApiAuth;
    pub use ::algo::{EntryPoint, DecodedEntryPoint, AlgoInput, AlgoOutput};
    pub use ::data::HasDataPath;
}

mod version;

static DEFAULT_API_BASE_URL: &'static str = "https://api.algorithmia.com";

/// The top-level struct for instantiating Algorithmia client endpoints
pub struct Algorithmia {
    http_client: Rc<HttpClient>,
}

impl<'a, 'c> Algorithmia {
    /// Instantiate a new client
    ///
    /// Client should be instatiated with your API key, except
    ///   when running within an algorithm on the Algorithmia platform.
    ///
    /// # Examples
    /// ```
    /// use algorithmia::*;
    /// // Initialize a client
    /// let client = Algorithmia::client("simUseYourApiKey");
    ///
    /// // Initialize a client (for algorithms running on the Algorithmia platform)
    /// let client = Algorithmia::client(ApiAuth::None);
    /// ```
    pub fn client<A: Into<ApiAuth>>(api_key: A) -> Algorithmia {
        let api_address = std::env::var("ALGORITHMIA_API").unwrap_or(DEFAULT_API_BASE_URL.into());
        Algorithmia { http_client: Rc::new(HttpClient::new(api_key.into(), &api_address)) }
    }

    /// Instantiate a new client against alternate API servers
    pub fn alt_client<A: Into<ApiAuth>, U: IntoUrl>(base_url: U, api_key: A) -> Algorithmia {
        Algorithmia { http_client: Rc::new(HttpClient::new(api_key.into(), base_url)) }
    }

    /// Instantiate an [`Algorithm`](algo/algorithm.struct.html) from this client
    ///
    /// By using In
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Algorithmia;
    /// use algorithmia::algo::Version;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let factor = client.algo("anowell/Dijkstra/0.1");
    /// let factor = client.algo(("anowell/Dijkstra", Version::Latest));
    /// ```
    pub fn algo<A: Into<AlgoRef>>(&self, algorithm: A) -> Algorithm {
        Algorithm::new(self.http_client.clone(), algorithm.into())
    }

    /// Instantiate a `DataDirectory` from this client
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let rustfoo = client.dir("data://.my/rustfoo");
    /// ```
    pub fn dir(&self, path: &'a str) -> DataDir {
        DataDir::new(self.http_client.clone(), path)
    }

    /// Instantiate a `DataDirectory` from this client
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let rustfoo = client.file("data://.my/rustfoo");
    /// ```
    pub fn file(&self, path: &'a str) -> DataFile {
        DataFile::new(self.http_client.clone(), path)
    }

    /// Instantiate a `DataPath` from this client
    ///
    /// Use this if you don't explicitly know if a Data URI is to a directory or file
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let rustfoo = client.data("data://.my/rustfoo/what_am_i");
    /// ```
    pub fn data(&self, path: &'a str) -> DataObject {
        DataObject::new(self.http_client.clone(), path)
    }
}


/// Allow cloning in order to reuse http client (and API key) for multiple connections
impl Clone for Algorithmia {
    fn clone(&self) -> Algorithmia {
        Algorithmia { http_client: self.http_client.clone() }
    }
}

impl Default for Algorithmia {
    fn default() -> Algorithmia {
        let api_address = std::env::var("ALGORITHMIA_API").unwrap_or(DEFAULT_API_BASE_URL.into());
        let api_key = std::env::var("ALGORITHMIA_API_KEY").map(ApiAuth::from).unwrap_or(ApiAuth::None);
        Algorithmia { http_client: Rc::new(HttpClient::new(api_key, &api_address)) }
    }
}