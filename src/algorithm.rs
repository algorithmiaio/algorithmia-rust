//! Algorithm module for executing Algorithmia algorithms
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Service;
//! use algorithmia::algorithm::{Algorithm, AlgorithmOutput, Version};
//!
//! // Initialize with an API key
//! let service = Service::new("111112222233333444445555566");
//! let factor = Algorithm::new("kenny", "Factor", Version::Revision(0,1,0));
//! let factor_service = factor.as_service(service);
//!
//! // Run the algorithm using a type safe decoding of the output to Vec<int>
//! //   since this algorithm outputs results as a JSON array of integers
//! let input = "19635".to_string();
//! let output: AlgorithmOutput<Vec<i64>> = factor_service.exec(&input).unwrap();
//! println!("Completed in {} seconds with result: {:?}", output.duration, output.result);
//! ```

extern crate hyper;

use ::{Service, AlgorithmiaError, API_BASE_URL};
use hyper::Url;
use rustc_serialize::{json, Decoder, Decodable, Encodable};
use std::io::Read;
use std::fmt;

static ALGORITHM_BASE_PATH: &'static str = "api";

/// Algorithmia algorithm
pub struct Algorithm<'a> {
    pub user: &'a str,
    pub repo: &'a str,
    pub version: Version<'a>,
}

/// Version of an algorithm
pub enum Version<'a> {
    /// Latest published version
    Latest,
    /// Latest published version with the same minor version, e.g., 1.2 implies 1.2.*
    Minor(u32, u32),
    /// A specific published revision, e.g., 0.1.0
    Revision(u32, u32, u32),
    /// A specific git hash - only works for the algorithm's author
    Hash(&'a str),
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
    pub algorithm: &'a Algorithm<'a>,
}

impl <'a> Version<'a> {
    /// Initialize a Version from a version string slice
    ///
    /// # Examples
    /// ```
    /// # use algorithmia::algorithm::Version;
    /// assert_eq!(Version::from_str("1.2").to_string(), Version::Minor(1,2).to_string());
    /// assert_eq!(Version::from_str("1.2.3").to_string(), Version::Revision(1,2,3).to_string());
    /// assert_eq!(Version::from_str("abc123").to_string(), Version::Hash("abc123").to_string());
    /// ```
    pub fn from_str(version: &'a str) -> Version<'a> {
        match version.split('.').map(|p| p.parse::<u32>()).collect() {
            Ok(parts) => {
                let ver_parts: Vec<u32> = parts;
                match &*ver_parts {
                    [major, minor, revision] => Version::Revision(major, minor, revision),
                    [major, minor] => Version::Minor(major, minor),
                    _ => panic!("Failed to parse version {}", version),
                }
            },
            _ => Version::Hash(version),
        }
    }

    /// Convert a Verion to string (uses its Display trait implementation)
    pub fn to_string(&self) -> String { format!("{}", self) }
}

impl<'a> Algorithm<'a> {
    /// Instantiate an algorithm from it's parts
    ///
    /// # Examples
    /// ```
    /// # use algorithmia::algorithm::{Algorithm, Version};
    /// let factor = Algorithm::new("kenny", "Factor", Version::Revision(0,1,0));
    /// ```
    pub fn new(user: &'a str, repo: &'a str, version: Version<'a>) -> Algorithm<'a> {
        Algorithm {
            user: user,
            repo: repo,
            version: version
        }
    }

    /// Instantiate an algorithm from the algorithm's URI
    ///
    /// # Examples
    /// ```
    /// # use algorithmia::algorithm::{Algorithm, Version};
    /// let factor = Algorithm::from_str("kenny/Factor/0.1");
    /// ```
    pub fn from_str(algo_uri: &'a str) -> Result<Algorithm<'a>, &'a str> {
        // TODO: strip optional 'algo://' prefix
        match &*algo_uri.split("/").collect::<Vec<_>>() {
            [user, algo, version] => Ok(Algorithm::new(user, algo, Version::from_str(version))),
            [user, algo] => Ok(Algorithm::new(user, algo, Version::Latest)),
            _ => Err("Invalid algorithm URI")
        }
    }

    /// Get the API Endpoint URL for a particular algorithm
    pub fn to_url(&self) -> Url {
        let url_string = match self.version {
            Version::Latest => format!("{}/{}/{}/{}", API_BASE_URL, ALGORITHM_BASE_PATH, self.user, self.repo),
            ref version => format!("{}/{}/{}/{}/{}", API_BASE_URL, ALGORITHM_BASE_PATH, self.user, self.repo, version),
        };
        Url::parse(&*url_string).unwrap()
    }

    pub fn as_service(&'a self, service: Service) -> AlgorithmService<'a> {
        AlgorithmService {
            service: service,
            algorithm: self,
        }
    }
}

impl<'c> AlgorithmService<'c> {
    /// Execute an algorithm with typed JSON response decoding
    ///
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
    /// # use algorithmia::algorithm::{Algorithm, AlgorithmOutput, Version};
    /// let service = Service::new("111112222233333444445555566");
    /// let factor = Algorithm::new("kenny", "Factor", Version::Latest);
    /// let factor_service = service.algorithm(&factor);
    /// let input = "19635".to_string();
    /// match factor_service.exec(&input) {
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
    /// # use algorithmia::algorithm::{Algorithm, Version};
    /// let algo_service = Service::new("111112222233333444445555566");
    /// let factor = Algorithm::new("kenny", "Factor", Version::Latest);
    /// let factor_service = algo_service.algorithm(&factor);
    ///
    /// let output = match factor_service.exec_raw("37") {
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

/// Displays Version values suitable for printing
impl <'a> fmt::Display for Version<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Version::Latest => write!(f, "latest"),
            Version::Minor(major, minor) => write!(f, "{}.{}", major, minor),
            Version::Revision(major, minor, revision) => write!(f, "{}.{}.{}", major, minor, revision),
            Version::Hash(hash) => write!(f, "{}", hash),
        }
    }
}


#[test]
fn test_latest_to_url() {
    let algorithm = Algorithm{ user: "kenny", repo: "Factor", version: Version::Latest };
    assert_eq!(algorithm.to_url().serialize(), format!("{}/api/kenny/Factor", API_BASE_URL));
}

#[test]
fn test_revision_to_url() {
    let algorithm = Algorithm{ user: "kenny", repo: "Factor", version: Version::Revision(0,1,0) };
    assert_eq!(algorithm.to_url().serialize(), format!("{}/api/kenny/Factor/0.1.0", API_BASE_URL));
}

#[test]
fn test_minor_to_url() {
    let algorithm = Algorithm{ user: "kenny", repo: "Factor", version: Version::Minor(0,1) };
    assert_eq!(algorithm.to_url().serialize(), format!("{}/api/kenny/Factor/0.1", API_BASE_URL));
}

#[test]
fn test_hash_to_url() {
    let algorithm = Algorithm{ user: "kenny", repo: "Factor", version: Version::Hash("abcdef123456") };
    assert_eq!(algorithm.to_url().serialize(), format!("{}/api/kenny/Factor/abcdef123456", API_BASE_URL));
}

#[test]
fn test_latest_string() {
    let version = Version::Latest;
    assert_eq!(version.to_string(), format!("{}", version));
    assert_eq!(&*version.to_string(), "latest");
}

#[test]
fn test_revision_string() {
    let version = Version::Revision(1,2,3);
    assert_eq!(version.to_string(), format!("{}", version));
    assert_eq!(&*version.to_string(), "1.2.3");
}

#[test]
fn test_minor_string() {
    let version = Version::Minor(1,2);
    assert_eq!(version.to_string(), format!("{}", version));
    assert_eq!(&*version.to_string(), "1.2");
}

#[test]
fn test_json_decoding() {
    let json_output = r#"{"duration":0.46739511,"result":[5,41]}"#;
    let expected = AlgorithmOutput{ duration: 0.46739511f32, result: [5, 41] };
    let decoded: AlgorithmOutput<Vec<i32>> = json::decode(json_output).unwrap();
    assert_eq!(expected.duration, decoded.duration);
    assert_eq!(expected.result, &*decoded.result);
}
