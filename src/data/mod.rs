pub use data::dir::DataDir;
pub use data::file::DataFile;
pub use hyper::client::Body;

use {Algorithmia, HttpClient};
use error::*;
use hyper::Url;
use hyper::status::StatusCode;

pub mod dir;
pub mod file;


static COLLECTION_BASE_PATH: &'static str = "v1/data";

header! {
    (XDataType, "X-Data-Type") => [String]
}

header! {
    (XErrorMessage, "X-Error-Message") => [String]
}

pub enum DataType {
    File,
    Dir,
}

pub enum DataObject {
    File(DataFile),
    Dir(DataDir),
}

// Shared by results for deleting both files and directories
#[derive(RustcDecodable, Debug)]
pub struct DeletedResult {
    pub deleted: u64,
}

pub trait HasDataPath {
    fn new(client: HttpClient, path: &str) -> Self;
    fn path(&self) -> &str;
    fn client(&self) -> &HttpClient;

    /// Get the API Endpoint URL for a particular data URI
    fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}", Algorithmia::get_base_url(), COLLECTION_BASE_PATH, self.path());
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
        format!("data://{}", self.path())
    }

    /// Get the parent off a given Data Object
    ///
    /// ```
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::HasDataPath;
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_file = client.file("data://.my/my_dir/my_file");
    /// assert_eq!(my_file.parent().unwrap().path(), ".my/my_dir");
    /// ```
    fn parent(&self) -> Option<DataDir>{
        match self.path().rsplitn(2, "/").nth(1) {
            Some(path) => Some(DataDir::new(self.client().clone(), &*path)),
            None => None
        }
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
        match self.path().rsplitn(2, "/").next() {
            Some(s) => Some(s.to_string()),
            None => None
        }
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

pub fn parse_data_uri(data_uri: &str) -> &str {
    match data_uri {
        p if p.starts_with("data://") => &p[7..],
        p if p.starts_with("/") => &p[1..],
        p => p,
    }
}

impl HasDataPath for DataPath {
    fn new(client: HttpClient, path: &str) -> Self { DataPath { client: client, path: parse_data_uri(path).to_string() } }
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
        match res.status {
            StatusCode::Ok => match res.headers.get::<XDataType>() {
                Some(dt) if &*dt.to_string() == "directory" => Ok(DataType::Dir),
                Some(dt) if &*dt.to_string() == "file"  => Ok(DataType::File),
                Some(dt) => Err(Error::DataTypeError(format!("Unknown DataType: {}", dt.to_string()))),
                None => Err(Error::DataTypeError("Unspecified DataType".to_string())),
            },
            status => {
                let msg = match res.headers.get::<XErrorMessage>()  {
                    Some(err_header) => format!("{}: {}", status, err_header),
                    None => format!("{}", status),
                };

                Err(ApiError{message: msg, stacktrace: None}.into())
            },
        }
    }


    pub fn into_type(&self) -> Result<DataObject, Error> {
        match try!(self.get_type()) {
            DataType::Dir => Ok(DataObject::Dir(self.into())),
            DataType::File => Ok(DataObject::File(self.into())),
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