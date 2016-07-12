use data::*;
use client::HttpClient;
use error::*;
use chrono::{UTC, TimeZone};

use hyper::status::StatusCode;

pub struct DataObject {
    path: String,
    client: HttpClient,
}

impl HasDataPath for DataObject {
    fn new(client: HttpClient, path: &str) -> Self { DataObject { client: client, path: parse_data_uri(path).to_string() } }
    fn path(&self) -> &str { &self.path }
    fn client(&self) -> &HttpClient { &self.client }
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
    pub fn get_type(&self) -> Result<DataType, Error> {
        let req = self.client.head(self.to_url());
        let res = try!(req.send());
        let metadata = try!(parse_headers(&res.headers));

        match res.status {
            StatusCode::Ok => Ok(metadata.data_type),
            status => Err(ApiError{message: status.to_string(), stacktrace: None}.into()),
        }
    }


    pub fn into_type(&self) -> Result<DataItem, Error> {
        let req = self.client.head(self.to_url());
        let res = try!(req.send());
        let metadata = try!(parse_headers(&res.headers));

        match metadata.data_type {
            DataType::Dir => Ok(DataItem::Dir(DataDirItem{
                dir: self.into()
            })),
            DataType::File => Ok(DataItem::File(DataFileItem{
                size: metadata.content_length.unwrap_or(0),
                last_modified: metadata.last_modified.unwrap_or_else(|| UTC.ymd(2015, 3, 14).and_hms(8, 0, 0)),
                file: self.into()
            })),
        }
    }

}


impl <'a> From<&'a DataObject> for DataDir {
    fn from(d: &'a DataObject) -> Self {
        DataDir::new(d.client().clone(), d.path())
    }
}

impl <'a> From<&'a DataObject> for DataFile {
    fn from(d: &'a DataObject) -> Self {
        DataFile::new(d.client().clone(), d.path())
    }
}