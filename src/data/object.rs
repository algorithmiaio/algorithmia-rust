use data::*;
use client::HttpClient;
use error::*;
use chrono::{UTC, TimeZone};
use hyper::status::StatusCode;
use super::{parse_headers, parse_data_uri};


/// Algorithmia data object (file or directory)
pub struct DataObject {
    path: String,
    client: HttpClient,
}

impl HasDataPath for DataObject {
    fn new(client: HttpClient, path: &str) -> Self {
        DataObject {
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

impl DataObject {
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
    pub fn get_type(&self) -> Result<DataType> {
        let req = try!(self.client.head(&self.path));
        let res = try!(req.send());

        match res.status {
            StatusCode::Ok => {
                let metadata = try!(parse_headers(&res.headers));
                Ok(metadata.data_type)
            }
            StatusCode::NotFound => Err(Error::NotFound(self.to_url().unwrap())),
            status => {
                Err(ApiError {
                        message: status.to_string(),
                        stacktrace: None,
                    }
                    .into())
            }
        }
    }

    /// Determine if a data URI is for a file or directory and convert into the appropriate type
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::{DataItem, HasDataPath};
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_obj = client.data("data://.my/some/path");
    /// match my_obj.into_type().ok().unwrap() {
    ///     DataItem::File(f) => println!("{} is a file", f.to_data_uri()),
    ///     DataItem::Dir(d) => println!("{} is a directory", d.to_data_uri()),
    /// }
    /// ```
    pub fn into_type(self) -> Result<DataItem> {
        let metadata = {
            let req = try!(self.client.head(&self.path));
            let res = try!(req.send());
            if res.status == StatusCode::NotFound {
                return Err(Error::NotFound(self.to_url().unwrap()));
            }
            try!(parse_headers(&res.headers))
        };

        match metadata.data_type {
            DataType::Dir => Ok(DataItem::Dir(DataDirItem { dir: self.into() })),
            DataType::File => {
                Ok(DataItem::File(DataFileItem {
                    size: metadata.content_length.unwrap_or(0),
                    last_modified: metadata.last_modified
                        .unwrap_or_else(|| UTC.ymd(2015, 3, 14).and_hms(8, 0, 0)),
                    file: self.into(),
                }))
            }
        }
    }
}


impl From<DataObject> for DataDir {
    fn from(d: DataObject) -> Self {
        let uri = d.to_data_uri();
        DataDir::new(d.client, &uri)
    }
}

impl From<DataObject> for DataFile {
    fn from(d: DataObject) -> Self {
        let uri = d.to_data_uri();
        DataFile::new(d.client, &uri)
    }
}
