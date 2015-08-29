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

use {Algorithmia, HttpClient};
use hyper::Url;
use rustc_serialize::{json, Decodable, Encodable};
use std::io::Read;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use super::result::{AlgoResult, JsonResult, AlgoOutput};
use error::{Error, ApiErrorResponse};

static ALGORITHM_BASE_PATH: &'static str = "v1/algo";

/// Algorithmia algorithm
pub struct Algorithm {
    pub path: String,
    client: HttpClient,
}

impl Algorithm {
    pub fn new(client: HttpClient, algo_uri: &str) -> Algorithm {
        let path = match algo_uri {
            p if p.starts_with("algo://") => &p[7..],
            p if p.starts_with("/") => &p[1..],
            p => p,
        };

        Algorithm {
            client: client,
            path: path.to_string(),
        }
    }

    /// Get the API Endpoint URL for this Algorithm
    pub fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}", Algorithmia::get_base_url(), ALGORITHM_BASE_PATH, self.path);
        Url::parse(&url_string).unwrap()
    }

    /// Get the Algorithmia algo URI for this Algorithm
    pub fn to_algo_uri(&self) -> String {
        format!("algo://{}", self.path)
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
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::algo::{Algorithm, AlgoOutput, Version};
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let moving_avg = client.algo("timeseries", "SimpleMovingAverage", Version::Minor(0,1));
    /// let input = (vec![0,1,2,3,15,4,5,6,7], 3);
    /// match moving_avg.pipe(&input) {
    ///     Ok(out) => {
    ///         let myVal: AlgoOutput<Vec<f64>> = out;
    ///         println!("{:?}", myVal.result);
    ///     },
    ///     Err(err) => println!("ERROR: {}", err),
    /// };
    /// ```
    pub fn pipe<D, E>(&self, input_data: &E) -> AlgoResult<D>
            where D: Decodable,
                  E: Encodable {
        let raw_input = try!(json::encode(input_data));
        let res_json = try!(self.pipe_raw(&raw_input, Mime(TopLevel::Application, SubLevel::Json, vec![])));

        // pipe_raw has already attempted to decode into ApiErrorResponse, so we can skip that here
        match json::decode::<AlgoOutput<D>>(&res_json) {
            Ok(result) => Ok(result),
            Err(err) => Err(Error::DecoderErrorWithContext(err, res_json)),
        }
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
    /// let output = match minmax.pipe_raw("[2,3,4]", "application/json".parse().unwrap()) {
    ///    Ok(result) => result,
    ///    Err(err) => panic!("{}", err),
    /// };
    pub fn pipe_raw(&self, input_data: &str, content_type: Mime) -> JsonResult {
        let req = self.client.post(self.to_url())
            .header(ContentType(content_type))
            .body(input_data);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        match res.status.is_success() {
            true => match json::decode::<ApiErrorResponse>(&res_json) {
                Ok(err_res) => Err(err_res.error.into()),
                Err(_) => Ok(res_json),
            },
            false => Err(Algorithmia::decode_to_error(res_json)),
        }
    }

}


#[cfg(test)]
mod tests {
    use Algorithmia;
    use algo::version::Version;

    fn mock_client() -> Algorithmia { Algorithmia::client("") }

    #[test]
    fn test_algo_without_version_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo_from_str("/anowell/Pinky");
        assert_eq!(algorithm.to_url().serialize(), format!("{}/v1/algo/anowell/Pinky", Algorithmia::get_base_url()));
    }

    #[test]
    fn test_algo_without_prefix_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo_from_str("anowell/Pinky/0.1.0");
        assert_eq!(algorithm.to_url().serialize(), format!("{}/v1/algo/anowell/Pinky/0.1.0", Algorithmia::get_base_url()));
    }

    #[test]
    fn test_algo_with_prefix_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo_from_str("algo://anowell/Pinky/0.1");
        assert_eq!(algorithm.to_url().serialize(), format!("{}/v1/algo/anowell/Pinky/0.1", Algorithmia::get_base_url()));
    }

    #[test]
    fn test_algo_typesafe_to_url() {
        let mock_client = mock_client();
        let algorithm = mock_client.algo("anowell", "Pinky", Version::Hash("abcdef123456"));
        assert_eq!(algorithm.to_url().serialize(), format!("{}/v1/algo/anowell/Pinky/abcdef123456", Algorithmia::get_base_url()));
    }
}
