use crate::data::*;
use crate::error::{ErrorKind, Result, ResultExt, ApiError};

use crate::client::HttpClient;
use reqwest::{Url, StatusCode, Response};

/// Trait used for types that can be represented with an Algorithmia Data URI
pub trait HasDataPath {
    #[doc(hidden)]
    fn new(client: HttpClient, path: &str) -> Self;
    #[doc(hidden)]
    fn path(&self) -> &str;
    #[doc(hidden)]
    fn client(&self) -> &HttpClient;

    /// Get the API Endpoint URL for a particular data URI
    fn to_url(&self) -> Result<Url> {
        let base_url = self.client()
            .base_url
            .as_ref()
            .map_err(|err| *err)
            .chain_err(|| ErrorKind::InvalidBaseUrl)?;
        let path = format!("{}/{}", super::DATA_BASE_PATH, self.path());
        base_url.join(&path).chain_err(|| ErrorKind::InvalidDataUri(self.to_data_uri()))
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
        let parts = self.path().splitn(2, '/').collect::<Vec<_>>();
        match parts.len() {
            1 => format!("{}://", parts[0]),
            _ => parts.join("://"),
        }
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
    fn parent(&self) -> Option<DataDir> {
        // Remove trailing slash and split
        let parts: Vec<&str> = self.path().split_terminator('/').collect();
        // Reformat using protocol while dropping last part
        let parent_uri = match parts.len() {
            0 | 1 => None,
            2 => Some(format!("{}://", parts[0])),
            len => Some(format!("{}://{}", parts[0], parts[1..(len - 1)].join("/"))),
        };
        // Initialize new DataDir from the parent_uri
        parent_uri.map(|uri| DataDir::new(self.client().clone(), &uri))
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
            .rsplitn(2, '/')
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
    fn exists(&self) -> Result<bool> {
        let url = self.to_url()?;
        let client = self.client();
        let req = client.head(url);

        let res: Response =
            req.send()
                .chain_err(|| {
                    ErrorKind::Http(format!("checking existence of '{}'", self.to_data_uri()))
                })?;
        match res.status() {
            StatusCode::OK => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            status => {
                let msg = match res.headers().get("X-Error-Message") {
                    Some(err_header) => format!("{}: {:?}", status, err_header),
                    None => format!("{}", status),
                };

                Err(ErrorKind::Api(ApiError {
                        message: msg,
                        stacktrace: None,
                    })
                    .into())
            }
        }
    }
}
