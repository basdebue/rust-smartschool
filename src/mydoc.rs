//! A virtual file system hosted on the server.

use crate::{
    error::{Error, Result},
    http,
    serde::Json,
    upload::UploadDirectory,
    Client,
};
use chrono::{DateTime, FixedOffset};
use futures::stream::TryStream;
pub use mime::Mime;
pub use reqwest::r#async::Chunk;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};
pub use uuid::Uuid;

/// Changes a folder's color and returns the modified folder.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
///
/// # Examples
///
/// Changes the color of the first listed favorited folder to yellow.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderColor, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Favorites).await?;
/// let folder = mydoc::change_folder_color(&client, folders[0].id(), FolderColor::Yellow).await?;
///
/// assert_eq!(folder.color(), FolderColor::Yellow);
/// ```
pub async fn change_folder_color(
    client: &Client<'_>,
    id: CustomFolderId,
    new_color: FolderColor,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("newColor", Json::FolderColor(new_color));

    let url = format!("{}/mydoc/api/v1/folders/{}/change-color", client.url(), id);
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Copies a file into the specified destination folder and returns the newly
/// created copy.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The source file doesn't exist.
/// * The destination is
///   [`FolderId::Favorites`](crate::mydoc::FolderId::Favorites) or
///   [`FolderId::Trashed`](crate::mydoc::FolderId::Trashed).
/// * The destination folder doesn't exist.
///
/// # Examples
///
/// Copies the first listed favorited file into the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Favorites).await?;
/// let file = mydoc::copy_file(&client, files[0].id(), FolderId::Root).await?;
///
/// assert_eq!(file.parent_id(), FolderId::Root);
/// ```
pub async fn copy_file<I: Into<FolderId>>(
    client: &Client<'_>,
    source: FileId,
    destination: I,
) -> Result<File> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/files/{}/copy", client.url(), source);
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Copies a folder into the specified destination folder and returns the newly
/// created copy.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The source folder doesn't exist.
/// * The destination is
///   [`FolderId::Favorites`](crate::mydoc::FolderId::Favorites) or
///   [`FolderId::Trashed`](crate::mydoc::FolderId::Trashed).
/// * The destination folder doesn't exist.
/// * The destination folder is the source folder itself.
///
/// # Examples
///
/// Copies the first listed favorited folder into the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Favorites).await?;
/// let folder = mydoc::copy_folder(&client, folders[0].id(), FolderId::Root).await?;
///
/// assert_eq!(folder.parent_id(), FolderId::Root);
/// ```
pub async fn copy_folder<I: Into<FolderId>>(
    client: &Client<'_>,
    source: CustomFolderId,
    destination: I,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/folders/{}/copy", client.url(), source);
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Creates a file from a template.
///
/// The extension of the template gets appended to the file name, unless the
/// file name already has that extension.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The destination folder doesn't exist.
/// * The file name contains an [illegal character](crate::mydoc::rename_file)
///   or starts with a `.`.
/// * The template doesn't exist.
///
/// # Examples
///
/// Creates an empty spreadsheet in the root folder.
/// Note that the resulting file name is `foo.xlsx`, **not** `foo.xlsx.xlsx`.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId, Template},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let file = mydoc::create_file_from_template(&client, FolderId::Root, Template::Excel, "foo.xlsx").await?;
///
/// assert_eq!(file.name(), "foo.xlsx");
/// ```
pub async fn create_file_from_template<I: Into<FolderId>>(
    client: &Client<'_>,
    parent_id: I,
    name: &str,
    template: Template<'_>,
) -> Result<File> {
    let mut form = HashMap::new();
    form.insert("fileName", Json::Str(name));
    form.insert("targetFolderId", Json::FolderId(parent_id.into()));
    form.insert("templateType", Json::Str(template.as_str()));
    if let Template::Custom(reference) = template {
        form.insert("templateReference", Json::Str(reference));
    }

    let url = format!("{}/mydoc/api/v1/files/createfromtemplate", client.url());
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Creates a folder in the specified parent folder and returns the newly
/// created folder.
///
/// A parenthesized number may be appended to the folder name to distinguish it
/// from other folders with the same name.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The parent folder doesn't exist.
/// * The folder name is [illegal](crate::mydoc::rename_file).
///
/// # Examples
///
/// Creates a new folder in the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderColor, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let folder = mydoc::create_folder(&client, FolderId::Root, "test", FolderColor::Yellow).await?;
///
/// assert_eq!(folder.color(), FolderColor::Yellow);
/// assert_eq!(folder.name(), "test");
/// assert_eq!(folder.parent_folder(), FolderId::Root);
/// ```
pub async fn create_folder<I: Into<FolderId>>(
    client: &Client<'_>,
    parent_id: I,
    name: &str,
    color: FolderColor,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("color", Json::FolderColor(color));
    form.insert("name", Json::Str(name));
    form.insert("parentId", Json::FolderId(parent_id.into()));

    let url = format!("{}/mydoc/api/v1/folders/", client.url());
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Permanently deletes a file from the virtual file system. If you want to
/// trash the file instead, use [`trash_file`](crate::mydoc::trash_file).
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
///
/// # Examples
///
/// Deletes the first listed file of the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// mydoc::delete_file(&client, files[0].id()).await?;
/// ```
pub async fn delete_file(client: &Client<'_>, id: FileId) -> Result<()> {
    let url = format!("{}/mydoc/api/v1/files/{}", client.url(), id);
    http::delete(client.http_client(), &url).await
}

/// Permanently deletes a folder from the virtual file system. If you want to
/// trash the folder instead, use [`trash_folder`](crate::mydoc::trash_folder).
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
///
/// # Examples
///
/// Deletes the first listed subfolder of the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// mydoc::delete_folder(&client, folders[0].id()).await?;
/// ```
pub async fn delete_folder(client: &Client<'_>, id: CustomFolderId) -> Result<()> {
    let url = format!("{}/mydoc/api/v1/folders/{}", client.url(), id);
    http::delete(client.http_client(), &url).await
}

/// Downloads a file and returns its contents as a non-blocking stream of
/// [`Chunk`](crate::mydoc::Chunk)s.
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
///
/// # Examples
///
/// Downloads the first listed file of the root folder and writes its contents
/// to a local file.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use futures::{future, stream::TryStreamExt};
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
/// use std::{fs::File, io::Write};
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root);
///
/// let file = File::create("foo")?;
/// mydoc::download_file(&client, files[0].id())
///     .await?
///     .try_for_each(|chunk| future::ready(file.write_all(&chunk)))
///     .await?;
/// ```
pub async fn download_file(
    client: &Client<'_>,
    id: FileId,
) -> Result<impl TryStream<Ok = Chunk, Error = Error>> {
    let url = format!("{}/mydoc/api/v1/files/{}/download", client.url(), id);
    http::get_as_stream(client.http_client(), &url).await
}

/// Downloads a file at a specific revision and returns its contents as a
/// non-blocking stream of [`Chunk`](crate::mydoc::Chunk)s.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The file doesn't exist.
/// * The revision doesn't exist or isn't associated with the file.
///
/// # Examples
///
/// Downloads the current revision of the first listed file of the root folder
/// and writes its contents to a local file.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use futures::{future, stream::TryStreamExt};
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
/// use std::{fs::File, io::Write};
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root);
///
/// let smsc_file = files[0];
/// let std_file = File::create("foo")?;
/// mydoc::download_revision(&client, smsc_file.id(), smsc_file.current_revision().id())
///     .await?
///     .try_for_each(|chunk| future::ready(file.write_all(&chunk)))
///     .await?;
/// ```
pub async fn download_revision(
    client: &Client<'_>,
    file_id: FileId,
    revision_id: RevisionId,
) -> Result<impl TryStream<Ok = Chunk, Error = Error>> {
    let url = format!(
        "{}/mydoc/api/v1/files/{}/revisions/{}/download",
        client.url(),
        file_id,
        revision_id
    );
    http::get_as_stream(client.http_client(), &url).await
}

/// Returns a vector of history entries representing the history of a file,
/// sorted by date in descending order. Nonexistent files produce an empty
/// vector.
///
/// # Examples
///
/// Prints the history of the first listed file of the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// let history = mydoc::get_file_history(&client, files[0].id()).await?;
///
/// println!("{:?}", history);
/// ```
pub async fn get_file_history(client: &Client<'_>, id: FileId) -> Result<Vec<HistoryEntry>> {
    let url = format!("{}/mydoc/api/v1/files/{}/history", client.url(), id);
    http::get_as_json(client.http_client(), &url).await
}

/// Returns a vector of file revisions in arbitrary order.
/// Nonexistent files produce an empty vector.
///
/// # Examples
///
/// Prints a list of revisions, sorted by date in ascending order, of the first
/// listed file of the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// let mut revisions = mydoc::get_file_revisions(&client, files[0].id()).await?;
///
/// revisions.sort_by(|one, two| Ord::cmp(one.date_created(), two.date_created());
/// println!("{:?}", revisions);
/// ```
pub async fn get_file_revisions(client: &Client<'_>, id: FileId) -> Result<Vec<Revision>> {
    let url = format!("{}/mydoc/api/v1/files/{}/revisions", client.url(), id);
    http::get_as_json(client.http_client(), &url).await
}

/// Returns the contents of a folder in arbitrary order.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
///
/// # Examples
///
/// Prints a list of files contained in the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
///
/// println!("{:?}", files);
/// ```
pub async fn get_folder_contents<I: Into<FolderId>>(
    client: &Client<'_>,
    id: I,
) -> Result<(Vec<File>, Vec<Folder>)> {
    let id = id.into();
    let url = if id == FolderId::Root {
        format!("{}/mydoc/api/v1/directory-listing", client.url())
    } else {
        format!("{}/mydoc/api/v1/directory-listing/{}", client.url(), id)
    };
    let response: GetFolderContents = http::get_as_json(client.http_client(), &url).await?;
    Ok((response.files, response.folders))
}

/// Returns a vector of history entries representing the history of a folder,
/// sorted by date in descending order. Nonexistent folders produce an empty
/// vector.
///
/// # Examples
///
/// Prints the history of the first listed subfolder of the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// let history = mydoc::get_folder_history(&client, folders[0].id()).await?;
///
/// println!("{:?}", history);
/// ```
pub async fn get_folder_history(
    client: &Client<'_>,
    id: CustomFolderId,
) -> Result<Vec<HistoryEntry>> {
    let url = format!("{}/mydoc/api/v1/folders/{}/history", client.url(), id);
    http::get_as_json(client.http_client(), &url).await
}

/// Returns the folder's path represented as a vector of breadcrumbs.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
///
/// # Examples
///
/// Creates a new folder tree in the root folder and looks up its path.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let folder_1 = mydoc::create_folder(&client, FolderId::Root, "foo").await?;
/// let folder_2 = mydoc::create_folder(&client, FolderId::Custom(folder_1.id()), "bar").await?;
/// let folder_3 = mydoc::create_folder(&client, FolderId::Custom(folder_2.id()), "baz").await?;
/// let parents = mydoc::get_folder_parents(&client, folder_3.id()).await?;
///
/// assert_eq!(&parents, &[folder_1.id(), folder_2.id()]);
/// ```
pub async fn get_folder_parents(
    client: &Client<'_>,
    id: CustomFolderId,
) -> Result<Vec<CustomFolderId>> {
    let url = format!("{}/mydoc/api/v1/folders/{}/parents", client.url(), id);
    http::get_as_json(client.http_client(), &url).await
}

/// Returns a vector of recently modified files, sorted by modification date in
/// descending order.
///
/// # Examples
///
/// Prints a list of recently modified files.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let files = mydoc::get_recent_files(&client).await?;
///
/// println!("{:?}", files);
/// ```
pub async fn get_recent_files(client: &Client<'_>) -> Result<Vec<File>> {
    let url = format!("{}/mydoc/api/v1/files/recent", client.url());
    http::get_as_json(client.http_client(), &url).await
}

/// Marks a file as favorite and returns the modified file.
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
///
/// # Examples
///
/// Marks the first listed file of the root folder as favorite.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// let file = mydoc::mark_file_as_favorite(&client, files[0].id()).await?;
///
/// assert_eq!(true, file.is_favorite());
/// ```
pub async fn mark_file_as_favorite(client: &Client<'_>, id: FileId) -> Result<File> {
    let url = format!(
        "{}/mydoc/api/v1/files/{}/mark-as-favourite",
        client.url(),
        id
    );
    http::post_as_json(client.http_client(), &url).await
}

/// Marks a folder as favorite and returns the modified folder.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
///
/// # Examples
///
/// Marks the first listed subfolder of the root folder as favorite.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// let folder = mydoc::mark_folder_as_favorite(&client, folders[0].id()).await?;
///
/// assert_eq!(true, folder.is_favorite());
/// ```
pub async fn mark_folder_as_favorite(client: &Client<'_>, id: CustomFolderId) -> Result<Folder> {
    let url = format!(
        "{}/mydoc/api/v1/folders/{}/mark-as-favourite",
        client.url(),
        id
    );
    http::post_as_json(client.http_client(), &url).await
}

/// Moves a file into the specified destination folder and returns the moved
/// file.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The source file doesn't exist.
/// * The destination is
///   [`FolderId::Favorites`](crate::mydoc::FolderId::Favorites) or
///   [`FolderId::Trashed`](crate::mydoc::FolderId::Trashed).
/// * The destination folder doesn't exist.
/// * The destination folder is the source file's current parent folder.
///
/// # Examples
///
/// Moves the first listed favorited file into the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Favorites).await?;
///
/// let mut file = files[0];
/// if file.parent_id() != FolderId::Root {
///     file = mydoc::move_file(&client, file.id(), FolderId::Root).await?;
/// }
/// assert_eq!(FolderId::Root, file.parent_id());
/// ```
pub async fn move_file<I: Into<FolderId>>(
    client: &Client<'_>,
    source: FileId,
    destination: I,
) -> Result<File> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/files/{}/move", client.url(), source);
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Moves a folder into the specified destination folder and returns the moved
/// folder.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The source folder doesn't exist.
/// * The destination is
///   [`FolderId::Favorites`](crate::mydoc::FolderId::Favorites) or
///   [`FolderId::Trashed`](crate::mydoc::FolderId::Trashed).
/// * The destination folder doesn't exist.
/// * The destination folder is the source folder's current parent folder.
/// * The destination folder is the source folder itself.
///
/// # Examples
///
/// Moves the first listed favorited folder into the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Favorites).await?;
///
/// let mut folder = folders[0];
/// if folder.parent_id() != FolderId::Root {
///     folder = mydoc::move_folder(&client, folder.id(), FolderId::Root).await?;
/// }
/// assert_eq!(FolderId::Root, folder.parent_id());
/// ```
pub async fn move_folder<I: Into<FolderId>>(
    client: &Client<'_>,
    source: CustomFolderId,
    destination: I,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/folders/{}/move", client.url(), source);
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Changes a file's name and returns the modified file.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The file doesn't exist.
/// * The new name contains `/`, `:`, `*`, `?`, `"`, `\\`, `<`, `>` or `|`.
/// * The new name starts or ends with a `.`.
/// * The new name is the same as the current name.
///
/// # Examples
///
/// Renames the first listed file of the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
///
/// let mut file = files[0];
/// if file.name() != "example.txt" {
///     file = mydoc::rename_file(&client, file.id(), "example.txt").await?;
/// }
/// assert_eq!("example.txt", file.name());
/// ```
pub async fn rename_file(client: &Client<'_>, id: FileId, new_name: &str) -> Result<File> {
    let mut form = HashMap::new();
    form.insert("newName", Json::Str(new_name));

    let url = format!("{}/mydoc/api/v1/files/{}/rename", client.url(), id);
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Changes a folder's name and returns the modified folder.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The folder doesn't exist.
/// * The new name is [illegal](crate::mydoc::rename_file).
/// * The new name is the same as the current name.
///
/// # Examples
///
/// Renames the first listed subfolder of the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
///
/// let mut folder = folders[0];
/// if folder.name() != "example" {
///     folder = mydoc::rename_folder(&client, folder.id(), "example").await?;
/// }
/// assert_eq!("example", folder.name());
/// ```
pub async fn rename_folder(
    client: &Client<'_>,
    id: CustomFolderId,
    new_name: &str,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("newName", Json::Str(new_name));

    let url = format!("{}/mydoc/api/v1/folders/{}/rename", client.url(), id);
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Restores a trashed file to an active folder and returns the restored file.
///
/// Restoring a file to a trashed folder permanently deletes the file, even
/// though this function's return value lists the trashed folder as its new
/// parent folder.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The source file isn't trashed.
/// * The source file doesn't exist.
/// * The destination is
///   [`FolderId::Favorites`](crate::mydoc::FolderId::Favorites) or
///   [`FolderId::Trashed`](crate::mydoc::FolderId::Trashed).
/// * The destination folder doesn't exist.
///
/// # Examples
///
/// Restores the first listed trashed file to the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId, State},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Trashed).await?;
/// let file = mydoc::restore_file(&client, files[0].id(), FolderId::Root).await?;
///
/// assert_eq!(State::Active, file.state());
/// ```
pub async fn restore_file<I: Into<FolderId>>(
    client: &Client<'_>,
    id: FileId,
    destination: I,
) -> Result<File> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/files/{}/restore", client.url(), id);
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Restores a trashed folder to an active folder and returns the restored
/// folder.
///
/// Restoring an active folder to another active folder moves the former akin to
/// [`move_folder`](crate::mydoc::move_folder). Restoring an active folder to
/// itself permanently deletes it. Restoring any folder to a trashed folder
/// permanently deletes the former.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The source folder doesn't exist.
/// * The destination folder doesn't exist.
///
/// # Examples
///
/// Restores the first listed trashed folder to the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId, State},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Trashed).await?;
/// let folder = mydoc::restore_folder(&client, folders[0].id(), FolderId::Root).await?;
///
/// assert_eq!(State::Active, folder.state());
/// ```
pub async fn restore_folder<I: Into<FolderId>>(
    client: &Client<'_>,
    id: CustomFolderId,
    destination: I,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/folders/{}/restore", client.url(), id);
    http::post_json_as_json(client.http_client(), &url, &form).await
}

/// Restores a file to the specified revision and returns the new revision.
///
/// The new revision and its ancestor only share their content; they have a
/// different identifier and are seperately listed in the revision history.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The file doesn't exist.
/// * The revision doesn't exist or isn't associated with the file.
///
/// # Examples
///
/// Restores the first listed file of the root folder to its current revision,
/// effectively duplicating the current revision.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId, State},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
///
/// let file = files[0];
/// let revision = mydoc::restore_revision(&client, file.id(), file.current_revision_id()).await?;
/// assert_eq!(file.id(), revision.file_id());
/// ```
pub async fn restore_revision(
    client: &Client<'_>,
    file_id: FileId,
    revision_id: RevisionId,
) -> Result<Revision> {
    let url = format!(
        "{}/mydoc/api/v1/files/{}/revisions/{}/restore",
        client.url(),
        file_id,
        revision_id
    );
    http::post_as_json(client.http_client(), &url).await
}

/// Moves a file to the [`Trashed`](crate::mydoc::FolderId::Trashed) folder.
/// If you want to permanently delete the file instead, use
/// [`delete_file`](crate::mydoc::delete_file).
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
///
/// # Examples
///
/// Trashes the first listed file of the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// mydoc::trash_file(&client, files[0].id()).await?;
/// ```
pub async fn trash_file(client: &Client<'_>, id: FileId) -> Result<()> {
    let url = format!("{}/mydoc/api/v1/files/{}/trash", client.url(), id);
    http::post(client.http_client(), &url).await
}

/// Moves a folder into the [`Trashed`](crate::mydoc::FolderId::Trashed) folder.
/// If you want to permanently delete the folder instead, use
/// [`delete_folder`](crate::mydoc::delete_folder).
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
///
/// # Examples
///
/// Trashes the first listed subfolder of the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// mydoc::trash_folder(&client, folders[0].id()).await?;
/// ```
pub async fn trash_folder(client: &Client<'_>, id: CustomFolderId) -> Result<()> {
    let url = format!("{}/mydoc/api/v1/folders/{}/trash", client.url(), id);
    http::post(client.http_client(), &url).await
}

/// Unmarks a file as favorite and returns the modified file.
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
///
/// # Examples
///
/// Unmarks the first listed file of the root folder as favorite.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (files, _) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// let file = mydoc::unmark_file_as_favorite(&client, files[0].id()).await?;
///
/// assert_eq!(false, file.is_favorite());
/// ```
pub async fn unmark_file_as_favorite(client: &Client<'_>, id: FileId) -> Result<File> {
    let url = format!(
        "{}/mydoc/api/v1/files/{}/unmark-as-favourite",
        client.url(),
        id,
    );
    http::post_as_json(client.http_client(), &url).await
}

/// Unmarks a folder as favorite and returns the modified folder.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
///
/// # Examples
///
/// Unmarks the first listed subfolder of the root folder as favorite.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
/// let (_, folders) = mydoc::get_folder_contents(&client, FolderId::Root).await?;
/// let folder = mydoc::unmark_folder_as_favorite(&client, folders[0].id()).await?;
///
/// assert_eq!(false, folder.is_favorite());
/// ```
pub async fn unmark_folder_as_favorite(client: &Client<'_>, id: CustomFolderId) -> Result<Folder> {
    let url = format!(
        "{}/mydoc/api/v1/folders/{}/unmark-as-favourite",
        client.url(),
        id,
    );
    http::post_as_json(client.http_client(), &url).await
}

/// Uploads the contents of an
/// [`UploadDirectory`](crate::upload::UploadDirectory) to a folder and returns
/// a list of uploaded files. The upload directory will retain its contents and
/// can be reused until it expires after a certain amount of time.
///
/// Uploading to a trashed folder does nothing.
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The destination folder doesn't exist.
/// * The upload directory is invalid.
///
/// # Examples
///
/// Uploads a text file to the root folder.
///
/// ```ignore
/// #![feature(async_await)]
///
/// use smartschool::{
///     mydoc::{self, FolderId},
///     upload::{self, File},
///     Client,
/// };
///
/// let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
///
/// let file = File::from_text("Hello World").build("example.txt");
/// let upload_dir = upload::get_upload_directory(&client).await?;
/// upload::upload_file(&client, upload_dir.clone(), file).await?;
/// let files = mydoc::upload(&client, FolderId::Root, &upload_dir).await?;
///
/// assert_eq!(1, files.len());
/// assert_eq!("example.txt", files[0].name());
/// ```
pub async fn upload<I: Into<FolderId>>(
    client: &Client<'_>,
    parent_id: I,
    upload_dir: &UploadDirectory,
) -> Result<Vec<File>> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(parent_id.into()));
    form.insert("uploadDir", Json::Str(upload_dir.as_str()));

    let url = format!("{}/mydoc/api/v1/files/upload", client.url());

    // The server response also contains an `exceptions` field, but this seems to
    // always be empty.
    let response: Upload = http::post_json_as_json(client.http_client(), &url, &form).await?;

    // The `files` field of the response is actually a map where the key is the
    // file's identifier and the value is the file itself. Since the files
    // themselves have an `id` field anyway, we discard the keys.
    Ok(response.files.into_iter().map(|(_, value)| value).collect())
}

/// A handle to a [`Folder`](crate::mydoc::Folder).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CustomFolderId(Uuid);

impl CustomFolderId {
    /// Returns the underlying [`Uuid`](uuid::Uuid).
    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for CustomFolderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Uuid> for CustomFolderId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// A file in the virtual file system.
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    current_revision: Revision,
    current_revision_id: RevisionId,
    date_changed: DateTime<FixedOffset>,
    date_created: DateTime<FixedOffset>,
    date_recent_action: DateTime<FixedOffset>,
    date_state_changed: DateTime<FixedOffset>,
    id: FileId,
    #[serde(rename = "isFavourite")]
    is_favorite: bool,
    name: String,
    parent_id: FolderId,
    state: State,
}

impl File {
    /// Returns a reference to the current revision of the file.
    pub fn current_revision(&self) -> &Revision {
        &self.current_revision
    }

    /// Returns the identifier of the file's current revision.
    pub fn current_revision_id(&self) -> RevisionId {
        self.current_revision_id
    }

    /// Returns the date when the file's content was last changed.
    pub fn date_changed(&self) -> DateTime<FixedOffset> {
        self.date_changed
    }

    /// Returns the date when the file was created.
    pub fn date_created(&self) -> DateTime<FixedOffset> {
        self.date_created
    }

    /// Returns the date when an action was last performed on the file.
    /// This includes actions that might not be immediately obvious, like
    /// downloading the file or marking the file as favorite.
    pub fn date_recent_action(&self) -> DateTime<FixedOffset> {
        self.date_recent_action
    }

    /// Returns the date when the file's state was last changed.
    pub fn date_state_changed(&self) -> DateTime<FixedOffset> {
        self.date_state_changed
    }

    /// Returns the file's identifier.
    pub fn id(&self) -> FileId {
        self.id
    }

    /// Consumes the file and returns its name.
    pub fn into_name(self) -> String {
        self.name
    }

    /// Consumes the file and returns its current revision.
    pub fn into_revision(self) -> Revision {
        self.current_revision
    }

    /// Returns `true` if the file is marked as favorite.
    pub fn is_favorite(&self) -> bool {
        self.is_favorite
    }

    /// Returns the name of the file.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the identifier of the file's parent folder.
    ///
    /// The returned [`FolderId`](crate::mydoc::FolderId) should only ever be of
    /// variants [`FolderId::Custom`](crate::mydoc::FolderId::Custom) or
    /// [`FolderId::Root`](crate::mydoc::FolderId::Root).
    ///
    /// Trashed files always list
    /// [`FolderId::Root`](crate::mydoc::FolderId::Root) as their parent folder.
    /// To check if a file is trashed, you can query its state instead.
    pub fn parent_id(&self) -> FolderId {
        self.parent_id
    }

    /// Returns the file's state.
    pub fn state(&self) -> State {
        self.state
    }
}

/// A handle to a [`File`](crate::mydoc::File).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct FileId(Uuid);

impl FileId {
    /// Returns the underlying [`Uuid`](uuid::Uuid).
    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Uuid> for FileId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// A user-created folder in the virtual file system.
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    color: FolderColor,
    date_changed: DateTime<FixedOffset>,
    date_created: DateTime<FixedOffset>,
    date_state_changed: DateTime<FixedOffset>,
    #[serde(rename = "hasSubFolders")]
    has_subfolders: bool,
    id: CustomFolderId,
    #[serde(rename = "isFavourite")]
    is_favorite: bool,
    name: String,
    parent_id: FolderId,
    state: State,
}

impl Folder {
    /// Returns the color of the folder.
    pub fn color(&self) -> FolderColor {
        self.color
    }

    /// Returns the date when the folder was last changed.
    pub fn date_changed(&self) -> DateTime<FixedOffset> {
        self.date_changed
    }

    /// Returns the date when the folder was created.
    pub fn date_created(&self) -> DateTime<FixedOffset> {
        self.date_created
    }

    /// Returns the date when the folder's state was last changed.
    pub fn date_state_changed(&self) -> DateTime<FixedOffset> {
        self.date_state_changed
    }

    /// Returns `true` if the folder has subfolders.
    pub fn has_subfolders(&self) -> bool {
        self.has_subfolders
    }

    /// Returns the folder's identifier.
    pub fn id(&self) -> CustomFolderId {
        self.id
    }

    /// Consumes the folder and returns its name.
    pub fn into_name(self) -> String {
        self.name
    }

    /// Returns `true` if the folder is marked as favorite.
    pub fn is_favorite(&self) -> bool {
        self.is_favorite
    }

    /// Returns the name of the folder.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the identifier of the folder's parent folder.
    ///
    /// The returned [`FolderId`](crate::mydoc::FolderId) should only ever be of
    /// variants [`FolderId::Custom`](crate::mydoc::FolderId::Custom) or
    /// [`FolderId::Root`](crate::mydoc::FolderId::Root).
    ///
    /// Trashed folders always list
    /// [`FolderId::Root`](crate::mydoc::FolderId::Root) as their parent folder.
    /// To check if a folder is trashed, you query use its state instead.
    pub fn parent_id(&self) -> FolderId {
        self.parent_id
    }

    /// Returns the folder's state.
    pub fn state(&self) -> State {
        self.state
    }
}

/// The color of a folder.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FolderColor {
    /// An aqua-colored folder.
    Aqua,
    /// A black-colored folder.
    Black,
    /// A blue-colored folder.
    Blue,
    /// A brown-colored folder.
    Brown,
    /// A green-colored folder.
    Green,
    /// An orange-colored folder.
    Orange,
    /// A pink-colored folder.
    Pink,
    /// A purple-colored folder.
    Purple,
    /// A red-colored folder.
    Red,
    /// A white-colored folder.
    White,
    /// A yellow-colored folder.
    Yellow,
}

impl Default for FolderColor {
    fn default() -> Self {
        FolderColor::Yellow
    }
}

/// An identifier of a folder in the virtual file system.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FolderId {
    /// A user-created folder in the virtual file system.
    Custom(CustomFolderId),
    /// A special folder containing files and folders marked as favorites for
    /// quick access.
    ///
    /// Note that this folder doesn't actually own any of its contents. It
    /// merely contains references to other places in the virtual file system.
    /// It is also treated as such; the
    /// [`parent_id`](crate::mydoc::File::parent_id) of a file contained in this
    /// folder points to the file's true parent folder.
    ///
    /// Smartschool internally refers to this folder as `favourites`.
    /// For consistency, this crate opts to use the American spelling wherever
    /// possible.
    Favorites,
    /// The root folder of the virtual file system.
    Root,
    /// A folder containing trashed files.
    ///
    /// After 30 days, files in this folder are permanently deleted.
    Trashed,
}

impl fmt::Display for FolderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FolderId::Custom(uuid) => fmt::Display::fmt(uuid, f),
            FolderId::Favorites => write!(f, "favourites"),
            FolderId::Root => write!(f, ""),
            FolderId::Trashed => write!(f, "trashed"),
        }
    }
}

impl From<CustomFolderId> for FolderId {
    fn from(id: CustomFolderId) -> Self {
        FolderId::Custom(id)
    }
}

#[derive(Deserialize)]
struct GetFolderContents {
    pub files: Vec<File>,
    pub folders: Vec<Folder>,
}

/// A history entry representing an action performed on a file or folder.
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    date: DateTime<FixedOffset>,
    is_download_event: bool,
    is_special_event: bool,
    text: String,
    user: HistoryEntryUser,
}

impl HistoryEntry {
    /// Returns the date when the recorded event happened.
    pub fn date(&self) -> DateTime<FixedOffset> {
        self.date
    }

    /// Consumes the history entry and returns a textual representation of the
    /// performed action.
    pub fn into_text(self) -> String {
        self.text
    }

    /// Consumes the history entry and returns a struct representing the user
    /// who performed the action.
    pub fn into_user(self) -> HistoryEntryUser {
        self.user
    }

    /// Returns `true` if the entry represents a "download event", like viewing
    /// the file or downloading the file.
    pub fn is_download_event(&self) -> bool {
        self.is_download_event
    }

    /// Returns `true` if the entry represents a "special event".
    /// TODO: Figure out what this means.
    pub fn is_special_event(&self) -> bool {
        self.is_special_event
    }

    /// Returns a textual representation of the performed action.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns a struct representing the user who performed the action.
    pub fn user(&self) -> &HistoryEntryUser {
        &self.user
    }
}

/// A user who performed an action on a file or folder.
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub struct HistoryEntryUser {
    #[serde(rename = "userIdentifier")]
    id: String,
    name: String,
    #[serde(rename = "userPictureHash")]
    picture_hash: String,
}

impl HistoryEntryUser {
    /// Returns the user's identifier, which seems to equal
    /// `"{school-id}_{user-id}_{account-id}"`.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Consumes the user and returns their identifier.
    pub fn into_id(self) -> String {
        self.id
    }

    /// Consumes the user and returns their name.
    pub fn into_name(self) -> String {
        self.name
    }

    /// Consumes the user and returns their picture hash.
    pub fn into_picture_hash(self) -> String {
        self.picture_hash
    }

    /// Returns the user's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the user's picture hash.
    pub fn picture_hash(&self) -> &str {
        &self.picture_hash
    }
}

/// A revision of a file in the virtual file system.
// The server response also contains a `location` field which seems to equal
// `{school-id}_{user-id}_{account-id}_{revision-id}`.
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    #[serde(rename = "dateCreated")]
    date: DateTime<FixedOffset>,
    file_id: FileId,
    #[serde(rename = "label")]
    file_name: String,
    file_size: u64,
    #[serde(rename = "mimeType", with = "mime_serde_shim")]
    file_type: Mime,
    id: RevisionId,
}

impl Revision {
    /// Returns the date when the revision was made.
    pub fn date(&self) -> DateTime<FixedOffset> {
        self.date
    }

    /// Returns the associated file's identifier.
    pub fn file_id(&self) -> FileId {
        self.file_id
    }

    /// Returns the name of the associated file.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Returns the size of the associated file in bytes.
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Returns the MIME type of the associated file.
    pub fn file_type(&self) -> &Mime {
        &self.file_type
    }

    /// Returns the revision's identifier.
    pub fn id(&self) -> RevisionId {
        self.id
    }
}

/// A handle to a [`Revision`](crate::mydoc::Revision).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RevisionId(Uuid);

impl RevisionId {
    /// Returns the underlying [`Uuid`](uuid::Uuid).
    pub fn as_inner(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for RevisionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Uuid> for RevisionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// The state of a file or folder in the virtual file system.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum State {
    /// An active file or folder.
    Active,
    /// A deleted file or folder.
    Deleted,
    /// A trashed file or folder.
    Trashed,
}

/// A template for a file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Template<'a> {
    /// A custom school-specific template, identified by a template identifier
    /// string.
    ///
    /// Fetching a template identifier string is currently not possible and must
    /// happen manually.
    Custom(&'a str),
    /// The default template for spreadsheets.
    Excel,
    /// The default template for presentations.
    Powerpoint,
    /// The default template for documents.
    Word,
}

impl<'a> Template<'a> {
    fn as_str(&self) -> &'static str {
        match self {
            Template::Custom(_) => "template",
            Template::Excel => "excel",
            Template::Powerpoint => "powerpoint",
            Template::Word => "word",
        }
    }
}

#[derive(Deserialize)]
struct Upload {
    pub files: HashMap<String, File>,
}
