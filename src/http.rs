//! HTTP-related utility functions.

use crate::error::{Error, Result};
use futures::{
    compat::{Future01CompatExt, Stream01CompatExt},
    TryFutureExt, TryStream, TryStreamExt,
};
use reqwest::r#async::{multipart::Form, Chunk, Client, RequestBuilder, Response};
use serde::{de::DeserializeOwned, ser::Serialize};

/// Sends an HTTP DELETE request.
pub async fn delete(client: &Client, url: &str) -> Result<()> {
    let request = client.delete(url);
    send(request).await?;
    Ok(())
}

/// Sends an HTTP GET request and returns the response, parsed as JSON.
pub async fn get_as_json<T: DeserializeOwned>(client: &Client, url: &str) -> Result<T> {
    let request = client.get(url);
    send_as_json(request).await
}

/// Sends an HTTP GET request and returns the response as a non-blocking stream
/// of chunks.
pub async fn get_as_stream(
    client: &Client,
    url: &str,
) -> Result<impl TryStream<Ok = Chunk, Error = Error>> {
    let request = client.get(url);
    let response = send(request).await?.into_body().compat().err_into();
    Ok(response)
}

pub async fn get_as_text(client: &Client, url: &str) -> Result<String> {
    let request = client.get(url);
    let response = send(request).await?.text().compat().await?;
    Ok(response)
}

/// Sends an HTTP POST request.
pub async fn post(client: &Client, url: &str) -> Result<()> {
    let request = client.post(url);
    send(request).await?;
    Ok(())
}

/// Sends an HTTP POST request and returns the response, parsed as JSON.
pub async fn post_as_json<T: DeserializeOwned>(client: &Client, url: &str) -> Result<T> {
    let request = client.post(url);
    send_as_json(request).await
}

/// Sends an HTTP POST request with a JSON body and returns the response, parsed
/// as JSON.
pub async fn post_json_as_json<T: DeserializeOwned, U: Serialize>(
    client: &Client,
    url: &str,
    form: &U,
) -> Result<T> {
    let request = client.post(url).json(form);
    send_as_json(request).await
}

/// Sends an HTTP POST request with a `multipart/form-data` body.
pub async fn post_multipart(client: &Client, url: &str, form: Form) -> Result<()> {
    let request = client.post(url).multipart(form);
    send(request).await?;
    Ok(())
}

/// Sends an HTTP request.
pub async fn send(request: RequestBuilder) -> Result<Response> {
    let response = request.send().compat().await?;
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        Err(Error::StatusCode(status))
    } else {
        Ok(response)
    }
}

/// Sends an HTTP request and returns the response, parsed as JSON.
async fn send_as_json<T: DeserializeOwned>(request: RequestBuilder) -> Result<T> {
    send(request).await?.json().compat().err_into().await
}
