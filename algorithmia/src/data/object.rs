use data::*;
use error::{ApiError, Result, ResultExt, ErrorKind};
use client::HttpClient;
use chrono::{Utc, TimeZone};
use reqwest::StatusCode;
use super::{parse_headers, parse_data_uri};


/// Algorithmia data object (file or directory)
pub struct DataObject {
    path: String,
    client: HttpClient,
}

impl HasDataPath for DataObject {
    #[doc(hidden)]
    fn new(client: HttpClient, path: &str) -> Self {
        DataObject {
            client: client,
            path: parse_data_uri(path).to_string(),
        }
    }
    #[doc(hidden)]
    fn path(&self) -> &str {
        &self.path
    }
    #[doc(hidden)]
    fn client(&self) -> &HttpClient {
        &self.client
    }
}

impl DataObject {
    /// Determine if a particular data URI is for a file or directory
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::{DataType, HasDataPath};
    /// # let client = Algorithmia::client("111112222233333444445555566");
    /// let my_obj = client.data("data://.my/some/path");
    /// match my_obj.get_type().ok().unwrap() {
    ///     DataType::File => println!("{} is a file", my_obj.to_data_uri()),
    ///     DataType::Dir => println!("{} is a directory", my_obj.to_data_uri()),
    /// }
    /// ```
    pub fn get_type(&self) -> Result<DataType> {
        let url = self.to_url()?;
        let mut req = self.client.head(url);
        let res = req.send()
            .chain_err(|| {
                ErrorKind::Http(format!("getting type of '{}'", self.to_data_uri()))
            })?;

        match res.status() {
            StatusCode::Ok => {
                let metadata = parse_headers(res.headers())?;
                Ok(metadata.data_type)
            }
            StatusCode::NotFound => Err(ErrorKind::NotFound(self.to_url().unwrap()).into()),
            status => {
                Err(
                    ErrorKind::Api(ApiError {
                        message: status.to_string(),
                        stacktrace: None,
                    }).into(),
                )
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
            let url = self.to_url()?;
            let mut req = self.client.head(url);
            let res = req.send()
                .chain_err(|| {
                    ErrorKind::Http(format!("getting type of '{}'", self.to_data_uri()))
                })?;
            if res.status() == StatusCode::NotFound {
                return Err(ErrorKind::NotFound(self.to_url().unwrap()).into());
            }
            parse_headers(res.headers())?
        };

        match metadata.data_type {
            DataType::Dir => Ok(DataItem::Dir(DataDirItem { dir: self.into() })),
            DataType::File => {
                Ok(DataItem::File(DataFileItem {
                    size: metadata.content_length.unwrap_or(0),
                    last_modified: metadata.last_modified
                        // Fallback to Algorithmia public launch date :-)
                        .unwrap_or_else(|| Utc.ymd(2015, 3, 14).and_hms(8, 0, 0)),
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
