//! Directory module for managing Algorithmia Data Directories
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Algorithmia;
//! use std::fs::File;
//!
//! let client = Algorithmia::client("111112222233333444445555566");
//! let my_dir = client.dir(".my/my_dir");
//!
//! my_dir.create();
//! my_dir.put_file("/path/to/file");
//! ```

extern crate hyper;
extern crate chrono;

use {Algorithmia, AlgorithmiaError, ApiErrorResponse, HttpClient};
use hyper::Url;
use hyper::status::StatusCode;
use rustc_serialize::{json, Decoder, Decodable};
use std::io::Read;
use std::fs::File;
use std::path::Path;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use self::chrono::{DateTime, UTC};
use super::{parse_data_uri, HasDataPath, FileAddedResult, FileAdded, DeletedResult, XDataType};
use std::error::Error;

/// Algorithmia Data Directory
pub struct DataDir {
    path: String,
    client: HttpClient,
}


pub type DirectoryShowResult = Result<DirectoryShow, AlgorithmiaError>;
pub type DirectoryCreatedResult = Result<(), AlgorithmiaError>;
pub type DirectoryDeletedResult = Result<DirectoryDeleted, AlgorithmiaError>;

#[derive(RustcDecodable, Debug)]
pub struct DirectoryUpdated {
    pub acl: Option<DataAcl>,
}


/// Response when deleting a new Directory
#[derive(RustcDecodable, Debug)]
pub struct DirectoryDeleted {
    // Omitting deleted.number and error.number for now
    pub result: DeletedResult,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct FolderEntry {
    pub name: String,
    pub acl: Option<DataAcl>,
}

#[derive(Debug)]
pub struct FileEntry {
    pub filename: String,
    pub size: u64,
    pub last_modified: DateTime<UTC>,
}

// Manual implemented Decodable: https://github.com/lifthrasiir/rust-chrono/issues/43
impl Decodable for FileEntry {
    fn decode<D: Decoder>(d: &mut D) -> Result<FileEntry, D::Error> {
        d.read_struct("root", 0, |d| {
            Ok(FileEntry{
                filename: try!(d.read_struct_field("filename", 0, |d| Decodable::decode(d))),
                size: try!(d.read_struct_field("size", 0, |d| Decodable::decode(d))),
                last_modified: {
                    let json_str: String = try!(d.read_struct_field("last_modified", 0, |d| Decodable::decode(d)));
                    match json_str.parse() {
                        Ok(datetime) => datetime,
                        Err(err) => return Err(d.error(err.description())),
                    }
                },
            })
        })
    }
}


#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct DataAcl {
    pub read: Vec<String>
}

/// Response when querying an existing Directory
#[derive(RustcDecodable, Debug)]
pub struct DirectoryShow {
    pub acl: Option<DataAcl>,
    pub folders: Option<Vec<FolderEntry>>,
    pub files: Option<Vec<FileEntry>>,
    pub marker: Option<String>,
}

fn get_directory(dir: &DataDir, marker: Option<String>) -> DirectoryShowResult {
    let url = match marker {
        Some(m) => Url::parse(&format!("{}?marker={}", dir.to_url(), m)).unwrap(),
        None => dir.to_url(),
    };

    let req = dir.client.get(url);

    let mut res = try!(req.send());

    if let Some(data_type) = res.headers.get::<XDataType>() {
        if "directory" != data_type.to_string() {
            return Err(AlgorithmiaError::DataTypeError(format!("Expected directory, Received {}", data_type)));
        }
    }

    let mut res_json = String::new();
    try!(res.read_to_string(&mut res_json));

    match json::decode::<DirectoryShow>(&res_json) {
        Ok(result) => Ok(result),
        Err(why) => match json::decode::<ApiErrorResponse>(&res_json) {
            Ok(err_res) => Err(AlgorithmiaError::AlgorithmiaApiError(err_res.error)),
            Err(_) => Err(AlgorithmiaError::DecoderErrorWithContext(why, res_json)),
        }
    }

}

impl HasDataPath for DataDir {
    fn new(client: HttpClient, path: &str) -> Self { DataDir { client: client, path: parse_data_uri(path).to_string() } }
    fn path(&self) -> &str { &self.path }
    fn client(&self) -> &HttpClient { &self.client }
}

impl DataDir {

    pub fn child<T: HasDataPath>(&self, filename: &str) -> T {
        T::new(self.client.clone(), &format!("{}/{}", self.to_data_uri(), filename))
    }

    /// Display Directory details if it exists
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir(".my/my_dir");
    /// match my_dir.show() {
    ///   Ok(dir) => println!("Files: {}", dir.files.unwrap().iter().map(|f| f.filename.clone()).collect::<Vec<_>>().connect(", ")),
    ///   Err(e) => println!("ERROR: {:?}", e),
    /// };
    /// ```
    pub fn show(&self) -> DirectoryShowResult {
        get_directory(&self, None)
    }


    // pub fn list(&self) -> DirectoryListing {

    // }

    /// Create a Directory
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir(".my/my_dir");
    /// match my_dir.create() {
    ///   Ok(_) => println!("Successfully created Directory"),
    ///   Err(e) => println!("ERROR creating Directory: {:?}", e),
    /// };
    /// ```
    pub fn create(&self) -> DirectoryCreatedResult {
        let url = self.parent().unwrap().to_url(); //TODO: don't unwrap this

        // TODO: complete abuse of this structure
        let input_data = FolderEntry {
            name: self.basename().unwrap().to_string(), //TODO: don't unwrap this
            acl: Some(DataAcl { read: vec![] }),
        };
        let raw_input = try!(json::encode(&input_data));

        // POST request
        let req = self.client.post(url)
            .header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .body(&raw_input);

        // Parse response
        let mut res = try!(req.send());

        match res.status {
            StatusCode::Ok | StatusCode::Created => Ok(()),
            _ => {
                let mut res_json = String::new();
                try!(res.read_to_string(&mut res_json));
                Err(Algorithmia::decode_to_error(res_json))
            }
        }
    }


    /// Delete a Directory
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir(".my/my_dir");
    /// match my_dir.delete(false) {
    ///   Ok(_) => println!("Successfully deleted Directory"),
    ///   Err(e) => println!("ERROR deleting Directory: {:?}", e),
    /// };
    /// ```
    pub fn delete(&self, force: bool) -> DirectoryDeletedResult {
        // DELETE request
        let url_string = format!("{}?force={}", self.to_url(), force.to_string());
        let url = Url::parse(&url_string).unwrap();
        let req = self.client.delete(url);

        // Parse response
        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Algorithmia::decode_to_result::<DirectoryDeleted>(res_json)
    }


    /// Upload a file to an existing Directory
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir(".my/my_dir");
    ///
    /// match my_dir.put_file("/path/to/file") {
    ///   Ok(response) => println!("Successfully uploaded to: {}", response.result),
    ///   Err(e) => println!("ERROR uploading file: {:?}", e),
    /// };
    /// ```
    pub fn put_file<P: AsRef<Path>>(&self, file_path: P) -> FileAddedResult {
        // FIXME: A whole lot of unwrap going on here...
        let path_ref = file_path.as_ref();
        let url_string = format!("{}/{}",
            self.to_url(),
            path_ref.file_name().unwrap().to_str().unwrap()
        );
        let url = Url::parse(&url_string).unwrap();

        let mut file = File::open(path_ref).unwrap();
        let req = self.client.put(url).body(&mut file);

        let mut res = try!(req.send());
        let mut res_json = String::new();
        try!(res.read_to_string(&mut res_json));

        Algorithmia::decode_to_result::<FileAdded>(res_json)
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use data::HasDataPath;
    use Algorithmia;

    fn mock_client() -> Algorithmia { Algorithmia::client("") }

    #[test]
    fn test_to_url() {
        let dir = DataDir::new(mock_client().http_client, "data://anowell/foo");
        assert_eq!(dir.to_url().serialize(), format!("{}/v1/data/anowell/foo", Algorithmia::get_base_url()));
    }

    #[test]
    fn test_to_data_uri() {
        let dir = DataDir::new(mock_client().http_client, "/anowell/foo");
        assert_eq!(dir.to_data_uri(), "data://anowell/foo".to_string());
    }

    #[test]
    fn test_parent() {
        let dir = DataDir::new(mock_client().http_client, "data://anowell/foo");
        let expected = DataDir::new(mock_client().http_client, "data://anowell");
        assert_eq!(dir.parent().unwrap().path, expected.path);
    }
}