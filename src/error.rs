//! Error handling functionality.

use reqwest::Error as ReqwestError;
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;

/// An error returned by the `smartschool` crate.
#[derive(Debug)]
pub enum Error {
    /// An error that occurred while performing I/O.
    Io(IoError),
    /// An error originating from the `reqwest` crate.
    Reqwest(ReqwestError),
    /// An error representing an unexpected response from the server.
    Response(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl From<ReqwestError> for Error {
    fn from(err: ReqwestError) -> Error {
        Error::Reqwest(err)
    }
}

impl StdError for Error {}

/// A specialized [`Result`](std::result::Result) type for Smartschool-related operations.
pub type Result<T> = std::result::Result<T, Error>;
