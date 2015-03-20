//! Algorithm module for executing Algorithmia algorithms
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Service;
//! use algorithmia::algorithm::AlgorithmOutput;
//!
//! // Initialize with an API key
//! let algo_service = Service::new("111112222233333444445555566");
//! let factor = algo_service.algorithm("kenny", "Factor");
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = "19635".to_string();
//! let output: AlgorithmOutput<Vec<i64>> = factor.exec(&input).unwrap();
//! println!("Completed in {} seconds with result: {:?}", output.duration, output.result);
//! ```

extern crate hyper;

use ::{Service, AlgorithmiaError, API_BASE_URL};
use hyper::Url;
use rustc_serialize::{json, Decoder, Decodable, Encodable};
use std::io::Read;

static ALGORITHM_BASE_PATH: &'static str = "api";

trait Version {
    pub fn to_string(&self) -> String
}

/// Algorithmia algorithm
pub struct Algorithm<'a> {
    pub user: &'a str,
    pub repo: &'a str,
    pub version: Option<Version>,
}

pub struct HashVersion<'a> {pub hash: &'a str}
pub struct MajorVersion {pub major: u32}
pub struct MinorVersion {pub major: u32, pub minor: u32}
pub struct SemanticVersion {pub major: u32, pub minor: u32, pub revision: u32}

impl Version for MajorVersion {
    pub fn to_string(&self) -> String { format!("{}", self.major) }
}

impl Version for MinorVersion {
    pub fn to_string(&self) -> String { format!("{}.{}", self.major, self.minor) }
}

impl Version for SemanticVersion {
    pub fn to_string(&self) -> String { format!("{}.{}.{}", self.major, self.minor, self.revision) }
}

impl Version for HashVersion<'a> {
    pub fn to_string(&self) -> String { self.hash.to_string() }
}

/// Major version only, e.g., "1"
impl Version for MajorVersion {
    fn new(major: u32) -> MajorVersion {
        MajorVersion {major: major}
    }
}

/// Minor version, irrespective of revision number, e.g., "1.2"
impl MinorVersion {
    fn new(major: u32, minor: u32) -> MinorVersion {
        MinorVersion {major: major, minor: minor}
    }
}

/// Full semantic version, e.g., "1.2.3"
impl SemanticVersion {
    fn new(major: u32, minor: u32, revision: u32) -> SemanticVersion {
        SemanticVersion {major: major, minor: minor, revision: revision}
    }
}

/// Git hash version, e.g., "28702984bcb5168461c4cdb7722289db5dd7e2bc"
impl HashVersion<'a> {
    fn new(hash: &'a str) -> MajorVersion {
        HashVersion {hash: hash}
    }
}

/// Result type for generic `AlgorithmOutput` when calling `exec`
pub type AlgorithmResult<T> = Result<AlgorithmOutput<T>, AlgorithmiaError>;
/// Result type for the raw JSON returned by calling `exec_raw`
pub type AlgorithmJsonResult = Result<String, hyper::HttpError>;

/// Generic struct for decoding an algorithm response JSON
#[derive(RustcDecodable, Debug)]
pub struct AlgorithmOutput<T> {
    pub duration: f32,
    pub result: T,
}

/// Service endpoint for executing algorithms
pub struct AlgorithmService<'a> {
    pub service: Service,
    pub algorithm: Algorithm<'a>,
}

pub mod version {
    /// Initialize a Version from a version string
    pub fn from_str(version: &str) -> Version {
        version.split('.').collect().map(|p| p.parse::<u32>()) match {
            [Ok(major), Ok(minor), Ok(revision)] => SemanticVersion::new(major, minor, revision),
            [Ok(major), Ok(minor)] => MinorVersion::new(major, minor),
            [Ok(major)] => MajorVersion::new(major),
            _ => HashVersion::new(&*version.to_string()),
        }
    }
}

impl<'a> Algorithm<'a> {
    /// Get the API Endpoint URL for a particular algorithm
    fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}/{}", API_BASE_URL, ALGORITHM_BASE_PATH, self.user, self.repo);
        Url::parse(&*url_string).unwrap()
    }
}

impl<'c> AlgorithmService<'c> {

    /// Instantiate `AlgorithmService` directly - alternative to `Service::algorithm`
    ///
    /// # Examples
    /// ```
    /// # use algorithmia::algorithm::AlgorithmService;
    /// let mut factor = AlgorithmService::new("111112222233333444445555566", "kenny", "Factor");
    /// ```
    pub fn new(api_key: &'c str, user: &'c str, repo: &'c str, version: Option<Version>) -> AlgorithmService<'c> {
        AlgorithmService {
            service: Service::new(api_key),
            algorithm: Algorithm{ user: user, repo: repo },
            version: version
        }
    }

    /// Execute an algorithm with type-safety
    ////
    /// input_data must be JSON-encodable
    ///     use `#[derive(RustcEncodable)]` for complex input
    ///
    /// You must explicitly specify the output type `T`
    ///     `exec` will attempt to decode the response into AlgorithmOutput<T>
    ///
    /// If decoding fails, it will attempt to decode into `ApiError`
    ///     and if that fails, it will error with `DecoderErrorWithContext`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::{Service, AlgorithmiaError};
    /// # use algorithmia::algorithm::AlgorithmOutput;
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let mut factor = algo_service.algorithm("kenny", "Factor");
    /// let input = "19635".to_string();
    /// match factor.exec(&input) {
    ///     Ok(out) => {
    ///         let myVal: AlgorithmOutput<Vec<i64>> = out;
    ///         println!("{:?}", myVal.result);
    ///     },
    ///     Err(AlgorithmiaError::ApiError(error)) => {
    ///         println!("API Error: {:?}", error)
    ///     },
    ///     Err(e) => println!("ERROR: {:?}", e),
    /// };
    /// ```
    pub fn exec<'a, D, E>(&'c self, input_data: &E) -> AlgorithmResult<D>
            where D: Decodable,
                  E: Encodable {
        let raw_input = try!(json::encode(input_data));
        let res_json = try!(self.exec_raw(&*raw_input));

        Service::decode_to_result::<AlgorithmOutput<D>>(res_json)
    }


    /// Execute an algorithm with with string input and receive the raw JSON response
    ///
    /// `exec` provides an encoding/decoding wrapper around this method
    ///
    /// TODO: Understand if we need to support NOT setting Content-Type to application/json
    ///     if the input isn't actually JSON
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use algorithmia::Service;
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let mut factor = algo_service.algorithm("kenny", "Factor");
    ///
    /// let output = match factor.exec_raw("37") {
    ///    Ok(result) => result,
    ///    Err(why) => panic!("{:?}", why),
    /// };
    pub fn exec_raw(&'c self, input_data: &str) -> AlgorithmJsonResult {
        let ref mut api_client = self.service.api_client();
        let req = api_client.post_json(self.algorithm.to_url())
            .body(input_data);

        let mut res = try!(req.send());
        let mut res_string = String::new();
        try!(res.read_to_string(&mut res_string));
        Ok(res_string)
    }

}

#[test]
fn test_to_url() {
    let algorithm = Algorithm{ user: "kenny", repo: "Factor" };
    assert_eq!(algorithm.to_url().serialize(), format!("{}/api/kenny/Factor", API_BASE_URL))
}

#[test]
fn test_json_decoding() {
    let json_output = r#"{"duration":0.46739511,"result":[5,41]}"#;
    let expected = AlgorithmOutput{ duration: 0.46739511f32, result: [5, 41] };
    let decoded: AlgorithmOutput<Vec<i32>> = json::decode(json_output).unwrap();
    assert_eq!(expected.duration, decoded.duration);
    assert_eq!(expected.result, decoded.result);
}
