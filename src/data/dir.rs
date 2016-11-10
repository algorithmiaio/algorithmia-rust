//! Directory module for managing Algorithmia Data Directories
//!
//! # Examples
//!
//! ```no_run
//! use algorithmia::Algorithmia;
//! use algorithmia::data::DataAcl;
//! use std::fs::File;
//!
//! let client = Algorithmia::client("111112222233333444445555566");
//! let my_dir = client.dir(".my/my_dir");
//!
//! my_dir.create(DataAcl::default());
//! my_dir.put_file("/path/to/file");
//! ```

use client::HttpClient;
use error::{self, Error};
use data::*;
use super::parse_data_uri;
use super::header::XDataType;
use ::json;

use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::vec::IntoIter;

use chrono::{DateTime, UTC};
use reqwest::header::ContentType;

#[cfg(feature="with-rustc-serialize")]
use rustc_serialize::{Decodable, Decoder};

/// Algorithmia Data Directory
pub struct DataDir {
    path: String,
    client: HttpClient,
}


#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable))]
#[derive(Debug)]
pub struct DirectoryUpdated {
    pub acl: Option<DataAcl>,
}


/// Response when deleting a new Directory
#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable))]
#[derive(Debug)]
pub struct DirectoryDeleted {
    // Omitting deleted.number and error.number for now
    pub result: DeletedResult,
}

#[cfg_attr(feature="with-serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable, RustcEncodable))]
#[derive(Debug)]
struct FolderItem {
    pub name: String,
    pub acl: Option<DataAcl>,
}

#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[derive(Debug)]
struct FileItem {
    pub filename: String,
    pub size: u64,
    pub last_modified: DateTime<UTC>,
}

// Manual implemented Decodable: https://github.com/lifthrasiir/rust-chrono/issues/43
#[cfg(feature="with-rustc-serialize")]
impl Decodable for FileItem {
    fn decode<D: Decoder>(d: &mut D) -> Result<FileItem, D::Error> {
        use std::error::Error;
        d.read_struct("root", 0, |d| {
            Ok(FileItem {
                filename: d.read_struct_field("filename", 0, |d| Decodable::decode(d))?,
                size: d.read_struct_field("size", 0, |d| Decodable::decode(d))?,
                last_modified: {
                    let json_str: String =
                        d.read_struct_field("last_modified", 0, |d| Decodable::decode(d))?;
                    match json_str.parse() {
                        Ok(datetime) => datetime,
                        Err(err) => return Err(d.error(err.description())),
                    }
                },
            })
        })
    }
}

/// ACL that indicates permissions for a `DataDirectory`
/// See also: [`ReadAcl`](enum.ReadAcl.html) enum to construct a `DataACL`
#[cfg_attr(feature="with-serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable, RustcEncodable))]
#[derive(Debug)]
pub struct DataAcl {
    pub read: Vec<String>,
}

pub enum ReadAcl {
    /// Readable only by owner
    Private,
    /// Readable by owner's algorithms (regardless of caller)
    MyAlgorithms,
    /// Readable by any user
    Public,
}

impl Default for DataAcl {
    fn default() -> Self {
        ReadAcl::MyAlgorithms.into()
    }
}

impl From<ReadAcl> for DataAcl {
    fn from(acl: ReadAcl) -> Self {
        match acl {
            ReadAcl::Private => DataAcl { read: vec![] },
            ReadAcl::MyAlgorithms => DataAcl { read: vec!["algo://.my/*".into()] },
            ReadAcl::Public => DataAcl { read: vec!["user://*".into()] },
        }
    }
}

/// Response when querying an existing Directory
#[cfg_attr(feature="with-serde", derive(Deserialize))]
#[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable))]
#[derive(Debug)]
struct DirectoryShow {
    pub acl: Option<DataAcl>,
    pub folders: Option<Vec<FolderItem>>,
    pub files: Option<Vec<FileItem>>,
    pub marker: Option<String>,
}


pub struct DirectoryListing<'a> {
    pub acl: Option<DataAcl>,
    dir: &'a DataDir,
    folders: IntoIter<FolderItem>,
    files: IntoIter<FileItem>,
    marker: Option<String>,
    query_count: u32,
}

impl<'a> DirectoryListing<'a> {
    fn new(dir: &'a DataDir) -> DirectoryListing<'a> {
        DirectoryListing {
            acl: None,
            dir: dir,
            folders: Vec::new().into_iter(),
            files: Vec::new().into_iter(),
            marker: None,
            query_count: 0,
        }
    }
}

impl<'a> Iterator for DirectoryListing<'a> {
    type Item = Result<DataItem, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.folders.next() {
            // Return folders first
            Some(d) => Some(Ok(DataItem::Dir(DataDirItem { dir: self.dir.child(&d.name) }))),
            None => {
                match self.files.next() {
                    // Return files second
                    Some(f) => {
                        Some(Ok(DataItem::File(DataFileItem {
                            size: f.size,
                            last_modified: f.last_modified,
                            file: self.dir.child(&f.filename),
                        })))
                    }
                    None => {
                        // Query if there is another page of files/folders
                        if self.query_count == 0 || self.marker.is_some() {
                            self.query_count += 1;
                            match get_directory(self.dir, self.marker.clone()) {
                                Ok(ds) => {
                                    self.folders = ds.folders.unwrap_or_else(Vec::new).into_iter();
                                    self.files = ds.files.unwrap_or_else(Vec::new).into_iter();
                                    self.marker = ds.marker;
                                    self.next()
                                }
                                Err(err) => Some(Err(err)),
                            }
                        } else {
                            None
                        }
                    }
                }
            }
        }
    }
}

fn get_directory(dir: &DataDir, marker: Option<String>) -> Result<DirectoryShow, Error> {
    let mut url = dir.to_url()?;
    if let Some(ref m) = marker {
        url.query_pairs_mut().append_pair("marker", m);
    }

    let req = dir.client.get(url)?;
    let mut res = req.send()?;

    if res.status().is_success() {
        if let Some(data_type) = res.headers().get::<XDataType>() {
            if "directory" != data_type.as_str() {
                return Err(Error::UnexpectedDataType("directory", data_type.to_string()));
            }
        }
    }

    let mut res_json = String::new();
    res.read_to_string(&mut res_json)?;

    if res.status().is_success() {
        json::decode_str(&res_json).map_err(|err| err.into())
    } else {
        Err(error::decode(&res_json))
    }
}

impl HasDataPath for DataDir {
    fn new(client: HttpClient, path: &str) -> Self {
        DataDir {
            client: client,
            path: parse_data_uri(path).to_string(),
        }
    }
    fn path(&self) -> &str {
        &self.path
    }
    fn client(&self) -> &HttpClient {
        &self.client
    }
}

impl DataDir {
    /// Display Directory details if it exists
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::{DataItem, HasDataPath};
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir(".my/my_dir");
    /// let dir_list = my_dir.list();
    /// for entry in dir_list {
    ///     match entry {
    ///         Ok(DataItem::File(f)) => println!("File: {}", f.to_data_uri()),
    ///         Ok(DataItem::Dir(d)) => println!("Dir: {}", d.to_data_uri()),
    ///         Err(err) => { println!("Error: {}", err); break; },
    ///     }
    /// };
    /// ```
    pub fn list(&self) -> DirectoryListing {
        DirectoryListing::new(self)
    }

    /// Create a Directory
    ///
    /// Use `DataAcl::default()` or the `ReadAcl` enum to set the ACL
    ///
    /// # Examples
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::DataAcl;
    /// let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir(".my/my_dir");
    /// match my_dir.create(DataAcl::default()) {
    ///   Ok(_) => println!("Successfully created Directory"),
    ///   Err(e) => println!("Error created directory: {}", e),
    /// };
    /// ```
    pub fn create<Acl: Into<DataAcl>>(&self, acl: Acl) -> Result<(), Error> {
        let parent = self.parent().ok_or_else(|| Error::InvalidDataPath(self.path.clone()))?;
        let parent_url = parent.to_url()?;

        // TODO: address complete abuse of this structure
        let input_data = FolderItem {
            name: self.basename()
                .ok_or_else(|| Error::InvalidDataPath(self.path.clone()))?
                .into(),
            acl: Some(acl.into()),
        };
        let raw_input = json::encode(&input_data)?;

        // POST request
        let req = self.client
            .post(parent_url)?
            .header(ContentType(mime!(Application / Json)))
            .body(raw_input);

        // Parse response
        let mut res = req.send()?;

        if res.status().is_success() {
            Ok(())
        } else {
            let mut res_json = String::new();
            res.read_to_string(&mut res_json)?;
            Err(error::decode(&res_json))
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
    ///   Err(err) => println!("Error deleting directory: {}", err),
    /// };
    /// ```
    pub fn delete(&self, force: bool) -> Result<DirectoryDeleted, Error> {
        // DELETE request
        let mut url = self.to_url()?;
        if force {
            url.query_pairs_mut().append_pair("force", "true");
        }

        let req = self.client.delete(url)?;

        // Parse response
        let mut res = req.send()?;
        let mut res_json = String::new();
        res.read_to_string(&mut res_json)?;

        if res.status().is_success() {
            json::decode_str(&res_json).map_err(|err| err.into())
        } else {
            Err(error::decode(&res_json))
        }
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
    ///   Err(err) => println!("Error uploading file: {}", err),
    /// };
    /// ```
    pub fn put_file<P: AsRef<Path>>(&self, file_path: P) -> Result<FileAdded, Error> {
        // FIXME: A whole lot of unwrap going on here...
        let path_ref = file_path.as_ref();
        let filename = path_ref.file_name().unwrap().to_str().unwrap();
        let file = File::open(path_ref)?;

        let data_file: DataFile = self.child(filename);
        data_file.put(file)
    }

    pub fn child<T: HasDataPath>(&self, filename: &str) -> T {
        let new_uri = match self.to_data_uri() {
            ref uri if uri.ends_with('/') => format!("{}{}", uri, filename),
            uri => format!("{}/{}", uri, filename),
        };
        T::new(self.client.clone(), &new_uri)
    }
}


#[cfg(test)]
mod tests {
    use data::HasDataPath;
    use super::*;
    use Algorithmia;

    fn mock_client() -> Algorithmia {
        Algorithmia::client("")
    }

    #[test]
    fn test_to_url() {
        let dir = mock_client().dir("data://anowell/foo");
        assert_eq!(dir.to_url().unwrap().path(),
                   "/v1/connector/data/anowell/foo");
    }

    #[test]
    fn test_to_data_uri() {
        let dir = mock_client().dir("/anowell/foo");
        assert_eq!(dir.to_data_uri(), "data://anowell/foo".to_string());
    }

    #[test]
    fn test_parent() {
        let dir = mock_client().dir("data://anowell/foo");
        let expected = mock_client().dir("data://anowell");
        assert_eq!(dir.parent().unwrap().path, expected.path);

        let dir = mock_client().dir("dropbox://anowell/foo");
        let expected = mock_client().dir("dropbox://anowell");
        assert_eq!(dir.parent().unwrap().path, expected.path);

        let dir = mock_client().dir("data://anowell");
        let expected = mock_client().dir("data://");
        assert_eq!(dir.parent().unwrap().path, expected.path);

        let dir = mock_client().dir("data://");
        assert!(dir.parent().is_none());
    }

    #[test]
    fn test_default_acl() {
        let acl: DataAcl = DataAcl::default();
        assert_eq!(acl.read, vec!["algo://.my/*".to_string()]);
    }

    #[test]
    fn test_private_acl() {
        let acl: DataAcl = ReadAcl::Private.into();
        assert!(acl.read.is_empty());
    }

    #[test]
    fn test_public_acl() {
        let acl: DataAcl = ReadAcl::Public.into();
        assert_eq!(acl.read, vec!["user://*".to_string()]);
    }

    #[test]
    fn test_myalgos_acl() {
        let acl: DataAcl = ReadAcl::MyAlgorithms.into();
        assert_eq!(acl.read, vec!["algo://.my/*".to_string()]);
    }

}
