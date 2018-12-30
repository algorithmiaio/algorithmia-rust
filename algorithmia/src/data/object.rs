use super::{parse_data_uri, parse_headers};
use crate::client::HttpClient;
use crate::data::*;
use crate::error::{ApiError, ErrorKind, Result, ResultExt};
use chrono::{TimeZone, Utc};
use reqwest::StatusCode;

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
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// # let client = Algorithmia::client("111112222233333444445555566")?;
    /// let my_obj = client.data("data://.my/some/path");
    /// match my_obj.get_type()? {
    ///     DataType::File => println!("{} is a file", my_obj.to_data_uri()),
    ///     DataType::Dir => println!("{} is a directory", my_obj.to_data_uri()),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_type(&self) -> Result<DataType> {
        let url = self.to_url()?;
        let req = self.client.head(url);
        let res = req
            .send()
            .chain_err(|| ErrorKind::Http(format!("getting type of '{}'", self.to_data_uri())))?;

        match res.status() {
            StatusCode::OK => {
                let metadata = parse_headers(res.headers())?;
                Ok(metadata.data_type)
            }
            StatusCode::NOT_FOUND => Err(ErrorKind::NotFound(self.to_url().unwrap()).into()),
            status => Err(ApiError::from(status.to_string()).into()),
        }
    }

    /// Determine if a data URI is for a file or directory and convert into the appropriate type
    ///
    /// ```no_run
    /// # use algorithmia::Algorithmia;
    /// # use algorithmia::data::{DataItem, HasDataPath};
    /// # fn main() -> Result<(), Box<std::error::Error>> {
    /// # let client = Algorithmia::client("111112222233333444445555566")?;
    /// let my_obj = client.data("data://.my/some/path");
    /// match my_obj.into_type()? {
    ///     DataItem::File(f) => println!("{} is a file", f.to_data_uri()),
    ///     DataItem::Dir(d) => println!("{} is a directory", d.to_data_uri()),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_type(self) -> Result<DataItem> {
        let metadata = {
            let url = self.to_url()?;
            let req = self.client.head(url);
            let res = req.send().chain_err(|| {
                ErrorKind::Http(format!("getting type of '{}'", self.to_data_uri()))
            })?;
            if res.status() == StatusCode::NOT_FOUND {
                return Err(ErrorKind::NotFound(self.to_url().unwrap()).into());
            }
            parse_headers(res.headers())?
        };

        match metadata.data_type {
            DataType::Dir => Ok(DataItem::Dir(DataDirItem { dir: self.into() })),
            DataType::File => {
                Ok(DataItem::File(DataFileItem {
                    size: metadata.content_length.unwrap_or(0),
                    last_modified: metadata
                        .last_modified
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