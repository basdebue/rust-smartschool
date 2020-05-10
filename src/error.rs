//! Error handling functionality.

use serde_json::Error as JsonError;
use serde_urlencoded::ser::Error as UrlEncodedError;
use std::{error::Error as StdError, fmt};
use surf::Exception as HttpError;

/// An error returned by the `smartschool` crate.
#[derive(Debug)]
pub enum Error {
    /// An authentication failure, most likely due to invalid login credentials.
    Authentication,
    /// An error returned by the [`surf`](surf) crate.
    Http(HttpError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<HttpError> for Error {
    fn from(err: HttpError) -> Self {
        Error::Http(err)
    }
}

// JSON deserialization errors are also returned as `surf::Exception`s, so we
// have opted to do the same for serialization errors.
impl From<JsonError> for Error {
    fn from(err: JsonError) -> Self {
        Error::Http(Box::new(err))
    }
}

// JSON errors are also returned as `surf::Exception`s, so we have opted to do
// the same for `x-www-form-urlencoded` errors.
impl From<UrlEncodedError> for Error {
    fn from(err: UrlEncodedError) -> Self {
        Error::Http(Box::new(err))
    }
}

impl StdError for Error {}

/// A specialized [`Result`](std::result::Result) type returned by the
/// `smartschool` crate.
pub type Result<T> = std::result::Result<T, Error>;
