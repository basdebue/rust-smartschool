//! HTTP-related utilities.

use crate::error::{Error, Result};
use bytes::Bytes;
use futures::{future::BoxFuture, stream, Stream};
use reqwest::{RequestBuilder, Response};

/// Unfolds a [`Response`](reqwest::Response) into a stream.
pub fn into_stream(response: Response) -> impl Stream<Item = Result<Bytes>> {
    stream::unfold(response, |mut response| {
        async {
            match response.chunk().await {
                Ok(Some(chunk)) => Some((Ok(chunk), response)),
                Ok(None) => None,
                Err(err) => Some((Err(err.into()), response)),
            }
        }
    })
}

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
