//! Error handling functionality.

use reqwest::{Error as ReqwestError, StatusCode};
use std::{error::Error as StdError, fmt};

/// An error returned by the `smartschool` crate.
#[derive(Debug)]
pub enum Error {
    /// An authentication failure, most likely due to invalid login credentials.
    Authentication,
    /// An error returned by the [`reqwest`](reqwest) crate.
    Reqwest(ReqwestError),
    /// An HTTP error response.
    StatusCode(StatusCode),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<ReqwestError> for Error {
    fn from(err: ReqwestError) -> Self {
        Error::Reqwest(err)
    }
}

impl StdError for Error {}

/// A specialized [`Result`](std::result::Result) type returned by the
/// `smartschool` crate.
pub type Result<T> = std::result::Result<T, Error>;
