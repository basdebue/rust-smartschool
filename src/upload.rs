//! File uploads for use around the platform.

use crate::{error::Result, http, Client};
use futures::stream::{TryStream, TryStreamExt};
pub use hyper::Chunk as UploadChunk;
use reqwest::r#async::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, error::Error};

/// Returns a handle to an empty upload destination.
///
/// The returned [`UploadDirectory`](crate::upload::UploadDirectory) is a
/// randomized hexadecimal string containing 30 characters.
///
/// # Examples
///
/// See [`mydoc::upload`](crate::mydoc::upload).
pub async fn get_upload_directory(client: &Client<'_>) -> Result<UploadDirectory> {
    let url = format!("{}/upload/api/v1/get-upload-directory", client.url());
    let response: GetUploadDirectory = http::get_as_json(client.http_client(), &url).await?;
    Ok(response.upload_dir)
}

/// Uploads a file to the specified upload directory.
///
/// The file name is not always kept intact:
///
/// * If the file name contains a `/` or a `\\`, all characters preceding this
///   character will be dropped.
/// * If the file name contains a different [illegal
///   character](crate::mydoc::rename_file), this character will be replaced
///   with a `_`.
///
/// # Errors
///
/// Returns an error if the file name contains a `:` or starts or ends with a
/// `.`.
///
/// # Examples
///
/// See [`mydoc::upload`](crate::mydoc::upload).
pub async fn upload_file(
    client: &Client<'_>,
    upload_dir: UploadDirectory,
    file: File,
) -> Result<()> {
    let form = Form::new()
        .text("uploadDir", upload_dir.inner)
        .part("file", file.inner);

    let url = format!("{}/Upload/Upload/Index", client.url());
    http::post_multipart(client.http_client(), &url, form).await
}

/// A file that can be uploaded.
pub struct File {
    inner: Part,
}

impl File {
    /// Creates a [`FileBuilder`](crate::upload::FileBuilder) from a collection
    /// of bytes.
    pub fn from_bytes<T>(bytes: T) -> FileBuilder
    where
        T: Into<Cow<'static, [u8]>>,
    {
        let inner = Part::bytes(bytes);
        FileBuilder { inner }
    }

    /// Creates a [`FileBuilder`](crate::upload::FileBuilder) from an
    /// asynchronous stream of [`UploadChunk`](crate::upload::UploadChunk)s.
    pub fn from_stream<T>(stream: T) -> FileBuilder
    where
        T: TryStream + Send + Unpin + 'static,
        T::Error: Error + Send + Sync,
        UploadChunk: From<T::Ok>,
    {
        let inner = Part::stream(stream.compat());
        FileBuilder { inner }
    }

    /// Creates a [`FileBuilder`](crate::upload::FileBuilder) from a string.
    pub fn from_text<T>(string: T) -> FileBuilder
    where
        T: Into<Cow<'static, str>>,
    {
        let inner = Part::text(string);
        FileBuilder { inner }
    }
}

/// A builder to construct the properties of a [`File`](crate::upload::File).
pub struct FileBuilder {
    inner: Part,
}

impl FileBuilder {
    /// Sets the file name and consumes the builder, returning a
    /// [`File`](crate::upload::File).
    pub fn build<T>(self, file_name: T) -> File
    where
        T: Into<Cow<'static, str>>,
    {
        let inner = self.inner.file_name(file_name);
        File { inner }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetUploadDirectory {
    pub upload_dir: UploadDirectory,
}

/// A handle to an upload destination.
///
/// *Any* nonempty UTF-8 string can be used as an upload directory.
/// Upload directories don't need to be fetched with
/// [`get_upload_directory`](crate::upload::get_upload_directory) to be valid.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct UploadDirectory {
    inner: Cow<'static, str>,
}

impl UploadDirectory {
    /// Returns a slice of the underlying string.
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<&'static str> for UploadDirectory {
    fn from(s: &'static str) -> UploadDirectory {
        UploadDirectory { inner: s.into() }
    }
}

impl From<String> for UploadDirectory {
    fn from(s: String) -> UploadDirectory {
        UploadDirectory { inner: s.into() }
    }
}
