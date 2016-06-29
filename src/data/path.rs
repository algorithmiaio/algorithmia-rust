use data::{self, DataFile, DataDir, DataType, DataObject, DataDirEntry, DataFileEntry, XErrorMessage};
use client::HttpClient;
use error::*;
use chrono::{UTC, TimeZone};

use hyper::Url;
use hyper::status::StatusCode;


pub trait HasDataPath {
    fn new(client: HttpClient, path: &str) -> Self;
    fn path(&self) -> &str;
    fn client(&self) -> &HttpClient;

    /// Get the API Endpoint URL for a particular data URI
    fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}", self.client().base_url, data::DATA_BASE_PATH, self.path());
        Url::parse(&url_string).unwrap()
    }

    /// Get the Algorithmia data URI a given Data Object
    ///
    /// ```
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::HasDataPath;
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir(".my/my_dir");
    /// assert_eq!(my_dir.to_data_uri(), "data://.my/my_dir");
    /// ```
    fn to_data_uri(&self) -> String {
        self.path().splitn(2, "/").collect::<Vec<_>>().join("://")
    }

    /// Get the parent off a given Data Object
    ///
    /// ```
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::HasDataPath;
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_file = client.file("data://.my/my_dir/my_file");
    /// assert_eq!(my_file.parent().unwrap().to_data_uri(), "data://.my/my_dir");
    /// ```
    fn parent(&self) -> Option<DataDir>{
        // Remove trailing slash and split
        let parts: Vec<&str> = self.path().split_terminator("/").collect();
        // Reformat using protocol while dropping last part
        let parent_uri = match parts.len() {
            0 | 1 => None,
            2 => Some(format!("{}://", parts[0])),
            len => Some(format!("{}://{}", parts[0], parts[1..(len-1)].join("/"))),
        };
        // Initialize new DataDir from the parent_uri
        parent_uri.map( |uri| DataDir::new(self.client().clone(), &uri))
    }

    /// Get the basename from the Data Object's path (i.e. unix `basename`)
    ///
    /// ```
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::HasDataPath;
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir("data:///.my/my_dir");
    /// assert_eq!(my_dir.basename().unwrap(), "my_dir");
    /// ```
    fn basename(&self) -> Option<String> {
        self.path()
            .rsplitn(2, "/")
            .next()
            .map(String::from)
    }


    /// Determine if a file or directory exists for a particular data URI
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::HasDataPath;
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_file = client.data("data://.my/my_dir/my_file");
    /// assert_eq!(my_file.exists().unwrap(), true);
    /// ```
    fn exists(&self) -> Result<bool, Error> {
        let req = self.client().head(self.to_url());

        let res = try!(req.send());
        match res.status {
            StatusCode::Ok => Ok(true),
            StatusCode::NotFound => Ok(false),
            status => {
                let msg = match res.headers.get::<XErrorMessage>()  {
                    Some(err_header) => format!("{}: {}", status, err_header),
                    None => format!("{}", status),
                };

                Err(ApiError{message: msg, stacktrace: None}.into())
            },
        }
    }
}

pub struct DataPath {
    path: String,
    client: HttpClient,
}


impl HasDataPath for DataPath {
    fn new(client: HttpClient, path: &str) -> Self { DataPath { client: client, path: data::parse_data_uri(path).to_string() } }
    fn path(&self) -> &str { &self.path }
    fn client(&self) -> &HttpClient { &self.client }
}

impl DataPath {

    /// Determine if a particular data URI is for a file or directory
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::{DataFile, DataDir, DataType, HasDataPath};
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_obj = client.data("data://.my/some/path");
    /// match my_obj.get_type().ok().unwrap() {
    ///     DataType::File => println!("{} is a file", my_obj.to_data_uri()),
    ///     DataType::Dir => println!("{} is a directory", my_obj.to_data_uri()),
    /// }
    /// ```
    pub fn get_type(&self) -> Result<DataType, Error> {
        let req = self.client.head(self.to_url());
        let res = try!(req.send());
        let metadata = try!(data::parse_headers(&res.headers));

        match res.status {
            StatusCode::Ok => Ok(metadata.data_type),
            status => Err(ApiError{message: status.to_string(), stacktrace: None}.into()),
        }
    }


    pub fn into_type(&self) -> Result<DataObject, Error> {
        let req = self.client.head(self.to_url());
        let res = try!(req.send());
        let metadata = try!(data::parse_headers(&res.headers));

        match metadata.data_type {
            DataType::Dir => Ok(DataObject::Dir(DataDirEntry{
                dir: self.into()
            })),
            DataType::File => Ok(DataObject::File(DataFileEntry{
                size: metadata.content_length.unwrap_or(0),
                last_modified: metadata.last_modified.unwrap_or_else(|| UTC.ymd(2015, 3, 14).and_hms(8, 0, 0)),
                file: self.into()
            })),
        }
    }

}


impl <'a> From<&'a DataPath> for DataDir {
    fn from(d: &'a DataPath) -> Self {
        DataDir::new(d.client().clone(), d.path())
    }
}

impl <'a> From<&'a DataPath> for DataFile {
    fn from(d: &'a DataPath) -> Self {
        DataFile::new(d.client().clone(), d.path())
    }
}