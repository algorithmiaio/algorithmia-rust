// //! Helpers to standardize decoding Algorithmia JSON responses

// use error::{Error};
// use rustc_serialize::{json, Decodable};

// pub fn decode<T: Decodable>(res_json: &str) -> Result<T, Error> {
//     match json::decode::<T>(res_json) {
//         Ok(result) => Ok(result),
//         Err(err) => Err(Error::DecoderErrorWithContext(err, res_json.into())),
//     }
// }




