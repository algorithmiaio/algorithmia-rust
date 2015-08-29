use error::Error;

/// Result type for generic `AlgoOutput` when calling `pipe`
pub type AlgoResult<T> = Result<AlgoOutput<T>, Error>;

/// Result type for the raw JSON returned by calling `pipe_raw`
pub type JsonResult = Result<String, Error>;

#[derive(RustcDecodable, Debug)]
pub struct AlgoMetadata {
    pub duration: f32
}

/// Generic struct for decoding an algorithm response JSON
#[derive(RustcDecodable, Debug)]
pub struct AlgoOutput<T> {
    pub metadata: AlgoMetadata,
    pub result: T,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_serialize::json;

    #[test]
    fn test_json_decoding() {
        let json_output = r#"{"metadata":{"duration":0.46739511},"result":[5,41]}"#;
        let expected = AlgoOutput{ metadata: AlgoMetadata { duration: 0.46739511f32} , result: [5, 41] };
        let decoded: AlgoOutput<Vec<i32>> = json::decode(json_output).unwrap();
        assert_eq!(expected.metadata.duration, decoded.metadata.duration);
        assert_eq!(expected.result, &*decoded.result);
    }
}
