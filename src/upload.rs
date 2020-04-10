//! File uploads for use around the platform.

use crate::{error::Result, Client};
use futures::AsyncRead;
use serde::{Deserialize, Serialize};

/// Returns a handle to an empty upload destination.
///
/// The returned [`UploadDirectory`](crate::upload::UploadDirectory) is a
/// randomized hexadecimal string containing 30 characters.
pub async fn get_upload_directory(client: &Client<'_>) -> Result<UploadDirectory> {
    let url = format!("{}/upload/api/v1/get-upload-directory", client.url());
    let GetUploadDirectory { upload_dir } = client.http_client().get(&url).recv_json().await?;
    Ok(upload_dir)
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
pub async fn upload_file(
    client: &Client<'_>,
    upload_dir: UploadDirectory,
    file: File,
) -> Result<()> {
    let form = Form::new()
        .text("uploadDir", upload_dir.inner)
        .part("file", file);

    let url = format!("{}/Upload/Upload/Index", client.url());
    client.http_client().post(&url).body(form).await?;
    Ok(())
}

/// A file that can be uploaded.
pub struct File;

impl File {
    /// Creates a [`FileBuilder`](crate::upload::FileBuilder) from a collection
    /// of bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> FileBuilder {
        unimplemented!();
    }

    /// Creates a [`FileBuilder`](crate::upload::FileBuilder) from an
    /// [asynchronous reader](futures::AsyncRead).
    pub fn from_reader(reader: impl AsyncRead) -> FileBuilder {
        unimplemented!();
    }

    /// Creates a [`FileBuilder`](crate::upload::FileBuilder) from a string.
    pub fn from_text(string: String) -> FileBuilder {
        unimplemented!();
    }
}

/// A builder to construct the properties of a [`File`](crate::upload::File).
pub struct FileBuilder;

impl FileBuilder {
    /// Sets the file name and consumes the builder, returning a
    /// [`File`](crate::upload::File).
    pub fn build<T>(self, file_name: String) -> File {
        unimplemented!();
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
    inner: String,
}

impl UploadDirectory {
    /// Returns a slice of the underlying string.
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<String> for UploadDirectory {
    fn from(s: String) -> UploadDirectory {
        UploadDirectory { inner: s.into() }
    }
}
