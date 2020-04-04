//! HTTP-related utilities.

use crate::error::{Error, Result};
use futures::future::BoxFuture;
use reqwest::{RequestBuilder, Response};

/// Adds a custom sending method to
/// [`RequestBuilder`](reqwest::RequestBuilder)s.
pub trait TrySend {
    // TODO: Use async trait method
    fn try_send(self) -> BoxFuture<'static, Result<Response>>;
}

impl TrySend for RequestBuilder {
    fn try_send(self) -> BoxFuture<'static, Result<Response>> {
        Box::pin(async {
            let response = self.send().await?;
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                Err(Error::StatusCode(status))
            } else {
                Ok(response)
            }
        })
    }
}
