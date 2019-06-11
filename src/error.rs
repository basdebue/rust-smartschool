//! Error handling functionality.

use reqwest::Error as ReqwestError;
use std::error::Error as StdError;
use std::fmt;

/// An error returned by the `smartschool` crate.
#[derive(Debug)]
pub enum Error {
    /// An error represening an invalid request by the client.
    Client(&'static str),
    /// An error originating from the `reqwest` crate.
    Reqwest(ReqwestError),
    /// An error representing an unexpected response from the server.
    Server(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<ReqwestError> for Error {
    fn from(err: ReqwestError) -> Error {
        Error::Reqwest(err)
    }
}

impl StdError for Error {}

/// A specialized [`Result`](std::result::Result) type returned by the `smartschool` crate.
pub type Result<T> = std::result::Result<T, Error>;
