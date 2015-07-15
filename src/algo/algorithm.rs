//! Algorithm module for executing Algorithmia algorithms
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
//! let output: AlgoOutput<Vec<f64>> = moving_avg.pipe(&input).unwrap();
//! println!("Completed in {} seconds with result: {:?}", output.metadata.duration, output.result);
//! ```

extern crate hyper;

use Algorithmia;
use hyper::Url;
use rustc_serialize::{json, Decodable, Encodable};
use std::io::Read;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use super::version::Version;
use super::result::{AlgoResult, JsonResult, AlgoOutput};

static ALGORITHM_BASE_PATH: &'static str = "v1/algo";

/// Algorithmia algorithm
pub struct Algorithm<'a> {
    pub client: Algorithmia,
    pub user: &'a str,
    pub repo: &'a str,
    pub version: Version<'a>,
}

impl<'a> Algorithm<'a> {
    /// Get the API Endpoint URL for a particular algorithm
    pub fn to_url(&self) -> Url {
        let url_string = match self.version {
            Version::Latest => format!("{}/{}/{}/{}", self.client.base_url, ALGORITHM_BASE_PATH, self.user, self.repo),
            ref version => format!("{}/{}/{}/{}/{}", self.client.base_url, ALGORITHM_BASE_PATH, self.user, self.repo, version),
        };
        Url::parse(&url_string).unwrap()
    }

    /// Execute an algorithm with typed JSON response decoding
    ///
    /// input_data must be JSON-encodable
    ///     use `#[derive(RustcEncodable)]` for complex input
    ///
    /// You must explicitly specify the output type `T`
    ///     `pipe` will attempt to decode the response into AlgoOutput<T>
    ///
    /// If decoding fails, it will attempt to decode into `ApiError`
    ///     and if that fails, it will error with `DecoderErrorWithContext`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::{Algorithmia, AlgorithmiaError};
    /// # use algorithmia::algo::{Algorithm, AlgoOutput, Version};
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let moving_avg = client.algo("timeseries", "SimpleMovingAverage", Version::Minor(0,1));
    /// let input = (vec![0,1,2,3,15,4,5,6,7], 3);
    /// match moving_avg.pipe(&input) {
    ///     Ok(out) => {
    ///         let myVal: AlgoOutput<Vec<f64>> = out;
    ///         println!("{:?}", myVal.result);
    ///     },
    ///     Err(e) => println!("ERROR: {:?}", e),
    /// };
    /// ```
    pub fn pipe<D, E>(&'a self, input_data: &E) -> AlgoResult<D>
            where D: Decodable,
                  E: Encodable {
        let raw_input = try!(json::encode(input_data));
        let res_json = try!(self.pipe_json(&raw_input, Mime(TopLevel::Application, SubLevel::Json, vec![])));

        Algorithmia::decode_to_result::<AlgoOutput<D>>(res_json)
    }


    /// pipeute an algorithm with with string input and receive the raw JSON response
    ///
    /// `pipe` provides a JSON encoding/decoding wrapper around this method
    ///
    /// TODO: Consider using byte slice input and output instead of strings
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::algo::{Algorithm, Version};
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let minmax  = client.algo("codeb34v3r", "FindMinMax", Version::Minor(0,1));
    ///
    /// let output = match minmax.pipe_json("[2,3,4]", "application/json".parse().unwrap()) {
    ///    Ok(result) => result,
    ///    Err(why) => panic!("{:?}", why),
    /// };
    pub fn pipe_json(&'a self, input_data: &str, content_type: Mime) -> JsonResult {
        let http_client = self.client.http_client();
        let req = http_client.post(self.to_url())
            .header(ContentType(content_type))
            .body(input_data);

        let mut res = try!(req.send());
        let mut res_string = String::new();
        try!(res.read_to_string(&mut res_string));
        Ok(res_string)
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use Algorithmia;
    use algo::version::Version;

    fn mock_client() -> Algorithmia { Algorithmia::client("") }

    #[test]
    fn test_latest_to_url() {
        let mock_client = mock_client();
        let algorithm = Algorithm {user: "kenny", repo: "Factor", version: Version::Latest, client: mock_client};
        assert_eq!(algorithm.to_url().serialize(), format!("{}/v1/algo/kenny/Factor", algorithm.client.base_url));
    }

    #[test]
    fn test_revision_to_url() {
        let mock_client = mock_client();
        let algorithm = Algorithm {user: "kenny", repo: "Factor", version: Version::Revision(0,1,0), client: mock_client};
        assert_eq!(algorithm.to_url().serialize(), format!("{}/v1/algo/kenny/Factor/0.1.0", algorithm.client.base_url));
    }

    #[test]
    fn test_minor_to_url() {
        let mock_client = mock_client();
        let algorithm = Algorithm {user: "kenny", repo: "Factor", version: Version::Minor(0,1), client: mock_client};
        assert_eq!(algorithm.to_url().serialize(), format!("{}/v1/algo/kenny/Factor/0.1", algorithm.client.base_url));
    }

    #[test]
    fn test_hash_to_url() {
        let mock_client = mock_client();
        let algorithm = Algorithm {user: "kenny", repo: "Factor", version: Version::Hash("abcdef123456"), client: mock_client};
        assert_eq!(algorithm.to_url().serialize(), format!("{}/v1/algo/kenny/Factor/abcdef123456", algorithm.client.base_url));
    }
}
