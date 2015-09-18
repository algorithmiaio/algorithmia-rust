//! Algorithmia client library
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Algorithmia;
//! use algorithmia::algo::{Algorithm, AlgoOutput, Version};
//!
//! // Initialize with an API key
//! let client = Algorithmia::client("111112222233333444445555566");
//! let moving_avg = client.algo("timeseries", "SimpleMovingAverage", Version::Minor(0,1));
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = (vec![0,1,2,3,15,4,5,6,7], 3);
//! let result: Vec<f64> = moving_avg.pipe(&input).unwrap().result().unwrap();
//! println!("Completed with result: {:?}", result);
//! ```

#![doc(html_logo_url = "https://algorithmia.com/assets/images/apple-touch-icon.png")]

#[macro_use]
extern crate hyper;
extern crate rustc_serialize;

use algo::{Algorithm, Version};
use data::{DataDir, DataFile, DataPath, HasDataPath};
use client::HttpClient;

use std::clone;

pub mod algo;
pub mod data;
pub mod error;
pub mod client;
pub mod json_helpers;
pub use hyper::{mime, Url};

static DEFAULT_API_BASE_URL: &'static str = "https://api.algorithmia.com";

/// The top-level struct for instantiating Algorithmia client endpoints
pub struct Algorithmia {
    http_client: HttpClient,
}

impl<'a, 'c> Algorithmia {
    /// Instantiate a new client
    pub fn client(api_key: &str) -> Algorithmia {
        Algorithmia {
            http_client: HttpClient::new(api_key.to_string(), DEFAULT_API_BASE_URL.to_string()),
        }
    }

    /// Instantiate a new client against alternate API servers
    pub fn alt_client(base_url: Url, api_key: &str) -> Algorithmia {
        Algorithmia {
            http_client: HttpClient::new(api_key.to_string(), base_url.serialize()),
        }
    }

    /// Instantiate an `Algorithm` from this client
    ///
    /// # Examples
    ///
    /// ```
    /// use algorithmia::Algorithmia;
    /// use algorithmia::algo::Version;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let factor = client.algo("anowell", "Dijkstra", Version::Latest);
    /// ```
    pub fn algo(&self, user: &str, algoname: &str, version: Version<'a>) -> Algorithm {
        let algo_uri = match version {
            Version::Latest => format!("{}/{}", user, algoname),
            ref ver => format!("{}/{}/{}", user, algoname, ver),
        };

        Algorithm::new(self.http_client.clone(), &*algo_uri)
    }

    /// Instantiate an `Algorithm` from this client using the algorithm's URI
    ///
    /// # Examples
    /// ```
    /// use algorithmia::Algorithmia;
    /// use algorithmia::algo::Version;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let factor = client.algo_from_str("anowell/Dijkstra/0.1");
    /// ```
    pub fn algo_from_str(&self, algo_uri: &str) -> Algorithm {
        Algorithm::new(self.http_client.clone(), algo_uri)
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
    pub fn data(&self, path: &'a str) -> DataPath {
        DataPath::new(self.http_client.clone(), path)
    }

}


/// Allow cloning in order to reuse http client (and API key) for multiple connections
impl clone::Clone for Algorithmia {
    fn clone(&self) -> Algorithmia {
        Algorithmia {
            http_client: self.http_client.clone(),
        }
    }
}
