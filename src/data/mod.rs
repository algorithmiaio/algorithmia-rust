pub use self::dir::DataDir;
pub use self::file::{DataFile, FileAddedResult, FileAdded};
use {AlgorithmiaError, ApiError};
use hyper::Url;
use hyper::status::StatusCode;
use Algorithmia;

mod dir;
mod file;


static COLLECTION_BASE_PATH: &'static str = "v1/data";

header! {
    (XDataType, "X-Data-Type") => [String]
}

header! {
    (XErrorMessage, "X-Error-Message") => [String]
}

#[derive(Debug)]
pub enum DataType {
    File,
    Directory,
    UnknownDataType(String),
}

// Shared by results for deleting both files and directories
#[derive(RustcDecodable, Debug)]
pub struct DeletedResult {
    pub deleted: u64,
}

pub struct DataObject {
    pub path: String,
    client: Algorithmia,
}

impl DataObject {
    fn new(client: Algorithmia, data_uri: &str) -> DataObject {
        let path = match data_uri {
            p if p.starts_with("data://") => p[7..].to_string(),
            p if p.starts_with("/") => p[1..].to_string(),
            p => p.to_string(),
        };

        DataObject {
            client: client,
            path: path
        }
    }

    /// Get the API Endpoint URL for a particular data URI
    pub fn to_url(&self) -> Url {
        let url_string = format!("{}/{}/{}", self.client.base_url, COLLECTION_BASE_PATH, self.path);
        Url::parse(&url_string).unwrap()
    }

    /// Get the Algorithmia data URI a given Data Object
    ///
    /// ```
    /// # use algorithmia::Algorithmia;
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir(".my/my_dir");
    /// assert_eq!(my_dir.to_data_uri(), "data://.my/my_dir");
    /// ```
    pub fn to_data_uri(&self) -> String {
        format!("data://{}", self.path)
    }

    /// Get the parent off a given Data Object
    ///
    /// ```
    /// # use algorithmia::Algorithmia;
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.file("data://.my/my_dir/my_file");
    /// assert_eq!(my_dir.parent().unwrap().path, ".my/my_dir");
    /// ```
    pub fn parent(&self) -> Option<DataDir>{
        match self.path.rsplitn(2, "/").nth(1) {
            Some(path) => Some(DataDir::new(self.client.clone(), &*path)),
            None => None
        }
    }

    /// Get the basename from the Data Object's path (i.e. unix `basename`)
    ///
    /// ```
    /// # use algorithmia::Algorithmia;
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_dir = client.dir("data:///.my/my_dir");
    /// assert_eq!(my_dir.basename().unwrap(), "my_dir");
    /// ```
    pub fn basename(&self) -> Option<String> {
        match self.path.rsplitn(2, "/").next() {
            Some(s) => Some(s.to_string()),
            None => None
        }
    }


    pub fn get_type(&self) -> Result<DataType, AlgorithmiaError> {
        let http_client = self.client.http_client();
        let req = http_client.head(self.to_url());

        let res = try!(req.send());
        match res.status {
            StatusCode::Ok => match res.headers.get::<XDataType>() {
                Some(dt) if &*dt.to_string() == "directory" => Ok(DataType::Directory),
                Some(dt) if &*dt.to_string() == "file"  => Ok(DataType::File),
                Some(dt) => Ok(DataType::UnknownDataType(dt.to_string())),
                None => Err(AlgorithmiaError::DataTypeError("Unspecified DataType".to_string())),
            },
            status => {
                let msg = match res.headers.get::<XErrorMessage>()  {
                    Some(err_header) => format!("{}: {}", status, err_header),
                    None => format!("{}", status),
                };

                Err(AlgorithmiaError::AlgorithmiaApiError(ApiError{message: msg, stacktrace: None}))
            },
        }
    }

    pub fn exists(&self) -> Result<bool, AlgorithmiaError> {
        let http_client = self.client.http_client();
        let req = http_client.head(self.to_url());

        let res = try!(req.send());
        match res.status {
            StatusCode::Ok => Ok(true),
            StatusCode::NotFound => Ok(false),
            status => {
                let msg = match res.headers.get::<XErrorMessage>()  {
                    Some(err_header) => format!("{}: {}", status, err_header),
                    None => format!("{}", status),
                };

                Err(AlgorithmiaError::AlgorithmiaApiError(ApiError{message: msg, stacktrace: None}))
            },
        }

    }

    pub fn is_dir(&self) -> Result<bool, AlgorithmiaError> {
        match self.get_type() {
            Ok(DataType::Directory) => Ok(true),
            Ok(_) => Ok(false),
            Err(err) => Err(err),
        }
    }
    pub fn is_file(&self) -> Result<bool, AlgorithmiaError> {
        match self.get_type() {
            Ok(DataType::File) => Ok(true),
            Ok(_) => Ok(false),
            Err(err) => Err(err),
        }
    }

}