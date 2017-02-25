
//! Implements the gerrit structure

use changes;
use url;

/// Interface for Gerrit access.
pub trait GerritAccess {
    /// Returns the (path, query) information
    fn build_url(&self) -> (String, String);
}


/// `Gerrit` structure for management of several gerrit endpoints
pub struct Gerrit {
    url: url::Url,
}

impl Gerrit {
    /// Creates a new `Gerrit` object
    ///
    /// The url points to the http endpoint of an gerrit server like
    /// `http://localhost:8080/gerrit`. All other function append to this url there endpoint pathes
    /// and query parameters.
    pub fn new<S>(url: S) -> Gerrit
    where S: Into<String> {
        Gerrit {
            url: url::Url::parse(&url.into()).unwrap(),
        }
    }

    /// Returnes a Change endpoint
    ///
    /// This represent a change endpoint for add, remove or manipulating of changes and changesets
    pub fn changes(&mut self) -> changes::Changes {
        changes::Changes::new(&self.url)
    }
}

