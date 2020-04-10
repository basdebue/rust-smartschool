//! A virtual file system hosted on the server.

use crate::{error::Result, serde::Json, upload::UploadDirectory, Client};
use chrono::{DateTime, FixedOffset};
use futures::{AsyncRead, TryFutureExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};
use uuid::Uuid;

/// Changes a folder's color and returns the modified folder.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
pub async fn change_folder_color(
    client: &Client<'_>,
    id: CustomFolderId,
    new_color: FolderColor,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("newColor", Json::FolderColor(new_color));

    let url = format!("{}/mydoc/api/v1/folders/{}/change-color", client.url(), id);
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
pub async fn copy_file<I: Into<FolderId>>(
    client: &Client<'_>,
    source: FileId,
    destination: I,
) -> Result<File> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/files/{}/copy", client.url(), source);
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
pub async fn copy_folder<I: Into<FolderId>>(
    client: &Client<'_>,
    source: CustomFolderId,
    destination: I,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/folders/{}/copy", client.url(), source);
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
}

/// Permanently deletes a file from the virtual file system. If you want to
/// trash the file instead, use [`trash_file`](crate::mydoc::trash_file).
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
pub async fn delete_file(client: &Client<'_>, id: FileId) -> Result<()> {
    let url = format!("{}/mydoc/api/v1/files/{}", client.url(), id);
    client.http_client().delete(&url).await?;
    Ok(())
}

/// Permanently deletes a folder from the virtual file system. If you want to
/// trash the folder instead, use [`trash_folder`](crate::mydoc::trash_folder).
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
pub async fn delete_folder(client: &Client<'_>, id: CustomFolderId) -> Result<()> {
    let url = format!("{}/mydoc/api/v1/folders/{}", client.url(), id);
    client.http_client().delete(&url).await?;
    Ok(())
}

/// Downloads a file and returns its contents as a non-blocking stream of
/// [`Bytes`](bytes::Bytes).
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
pub async fn download_file(client: &Client<'_>, id: FileId) -> Result<impl AsyncRead> {
    let url = format!("{}/mydoc/api/v1/files/{}/download", client.url(), id);
    client.http_client().get(&url).err_into().await
}

/// Downloads a file at a specific revision and returns its contents as a
/// non-blocking stream of [`Bytes`](bytes::Bytes).
///
/// # Errors
///
/// Returns an error in the following situations:
///
/// * The file doesn't exist.
/// * The revision doesn't exist or isn't associated with the file.
pub async fn download_revision(
    client: &Client<'_>,
    file_id: FileId,
    revision_id: RevisionId,
) -> Result<impl AsyncRead> {
    let url = format!(
        "{}/mydoc/api/v1/files/{}/revisions/{}/download",
        client.url(),
        file_id,
        revision_id
    );
    client.http_client().get(&url).err_into().await
}

/// Returns a vector of history entries representing the history of a file,
/// sorted by date in descending order. Nonexistent files produce an empty
/// vector.
pub async fn get_file_history(client: &Client<'_>, id: FileId) -> Result<Vec<HistoryEntry>> {
    let url = format!("{}/mydoc/api/v1/files/{}/history", client.url(), id);
    client.http_client().get(&url).recv_json().err_into().await
}

/// Returns a vector of file revisions in arbitrary order. Nonexistent files
/// produce an empty vector.
pub async fn get_file_revisions(client: &Client<'_>, id: FileId) -> Result<Vec<Revision>> {
    let url = format!("{}/mydoc/api/v1/files/{}/revisions", client.url(), id);
    client.http_client().get(&url).recv_json().err_into().await
}

/// Returns the contents of a folder in arbitrary order.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
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
    let GetFolderContents { files, folders } = client.http_client().get(&url).recv_json().await?;
    Ok((files, folders))
}

/// Returns a vector of history entries representing the history of a folder,
/// sorted by date in descending order. Nonexistent folders produce an empty
/// vector.
pub async fn get_folder_history(
    client: &Client<'_>,
    id: CustomFolderId,
) -> Result<Vec<HistoryEntry>> {
    let url = format!("{}/mydoc/api/v1/folders/{}/history", client.url(), id);
    client.http_client().get(&url).recv_json().err_into().await
}

/// Returns the folder's path represented as breadcrumbs, consisting of a vector
/// of folder identifiers.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
pub async fn get_folder_parents(
    client: &Client<'_>,
    id: CustomFolderId,
) -> Result<Vec<CustomFolderId>> {
    let url = format!("{}/mydoc/api/v1/folders/{}/parents", client.url(), id);
    client.http_client().get(&url).recv_json().err_into().await
}

/// Returns a vector of recently modified files, sorted by modification date in
/// descending order.
pub async fn get_recent_files(client: &Client<'_>) -> Result<Vec<File>> {
    let url = format!("{}/mydoc/api/v1/files/recent", client.url());
    client.http_client().get(&url).recv_json().err_into().await
}

/// Marks a file as favorite and returns the modified file.
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
pub async fn mark_file_as_favorite(client: &Client<'_>, id: FileId) -> Result<File> {
    let url = format!(
        "{}/mydoc/api/v1/files/{}/mark-as-favourite",
        client.url(),
        id
    );
    client.http_client().post(&url).recv_json().err_into().await
}

/// Marks a folder as favorite and returns the modified folder.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
pub async fn mark_folder_as_favorite(client: &Client<'_>, id: CustomFolderId) -> Result<Folder> {
    let url = format!(
        "{}/mydoc/api/v1/folders/{}/mark-as-favourite",
        client.url(),
        id
    );
    client.http_client().post(&url).recv_json().err_into().await
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
pub async fn move_file<I: Into<FolderId>>(
    client: &Client<'_>,
    source: FileId,
    destination: I,
) -> Result<File> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/files/{}/move", client.url(), source);
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
pub async fn move_folder<I: Into<FolderId>>(
    client: &Client<'_>,
    source: CustomFolderId,
    destination: I,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/folders/{}/move", client.url(), source);
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
pub async fn rename_file(client: &Client<'_>, id: FileId, new_name: &str) -> Result<File> {
    let mut form = HashMap::new();
    form.insert("newName", Json::Str(new_name));

    let url = format!("{}/mydoc/api/v1/files/{}/rename", client.url(), id);
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
/// ```
pub async fn rename_folder(
    client: &Client<'_>,
    id: CustomFolderId,
    new_name: &str,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("newName", Json::Str(new_name));

    let url = format!("{}/mydoc/api/v1/folders/{}/rename", client.url(), id);
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
pub async fn restore_file<I: Into<FolderId>>(
    client: &Client<'_>,
    id: FileId,
    destination: I,
) -> Result<File> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/files/{}/restore", client.url(), id);
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
pub async fn restore_folder<I: Into<FolderId>>(
    client: &Client<'_>,
    id: CustomFolderId,
    destination: I,
) -> Result<Folder> {
    let mut form = HashMap::new();
    form.insert("parentId", Json::FolderId(destination.into()));

    let url = format!("{}/mydoc/api/v1/folders/{}/restore", client.url(), id);
    client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .err_into()
        .await
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
    client.http_client().post(&url).recv_json().err_into().await
}

/// Moves a file to the [`Trashed`](crate::mydoc::FolderId::Trashed) folder.
/// If you want to permanently delete the file instead, use
/// [`delete_file`](crate::mydoc::delete_file).
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
pub async fn trash_file(client: &Client<'_>, id: FileId) -> Result<()> {
    let url = format!("{}/mydoc/api/v1/files/{}/trash", client.url(), id);
    client.http_client().post(&url).await?;
    Ok(())
}

/// Moves a folder into the [`Trashed`](crate::mydoc::FolderId::Trashed) folder.
/// If you want to permanently delete the folder instead, use
/// [`delete_folder`](crate::mydoc::delete_folder).
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
pub async fn trash_folder(client: &Client<'_>, id: CustomFolderId) -> Result<()> {
    let url = format!("{}/mydoc/api/v1/folders/{}/trash", client.url(), id);
    client.http_client().post(&url).await?;
    Ok(())
}

/// Unmarks a file as favorite and returns the modified file.
///
/// # Errors
///
/// Returns an error if the file doesn't exist.
pub async fn unmark_file_as_favorite(client: &Client<'_>, id: FileId) -> Result<File> {
    let url = format!(
        "{}/mydoc/api/v1/files/{}/unmark-as-favourite",
        client.url(),
        id,
    );
    client.http_client().post(&url).recv_json().err_into().await
}

/// Unmarks a folder as favorite and returns the modified folder.
///
/// # Errors
///
/// Returns an error if the folder doesn't exist.
pub async fn unmark_folder_as_favorite(client: &Client<'_>, id: CustomFolderId) -> Result<Folder> {
    let url = format!(
        "{}/mydoc/api/v1/folders/{}/unmark-as-favourite",
        client.url(),
        id,
    );
    client.http_client().post(&url).recv_json().err_into().await
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
    let Upload { files } = client
        .http_client()
        .post(&url)
        .body_json(&form)?
        .recv_json()
        .await?;

    // The `files` field of the response is actually a map where the key is the
    // file's identifier and the value is the file itself. Since the files
    // themselves have an `id` field anyway, we discard the keys.
    Ok(files.into_iter().map(|(_, value)| value).collect())
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
    /// The file's current revision.
    pub current_revision: Revision,
    /// The identifier of the file's current revision.
    pub current_revision_id: RevisionId,
    /// The date when the file's content was last changed.
    pub date_changed: DateTime<FixedOffset>,
    /// The date when the file was created.
    pub date_created: DateTime<FixedOffset>,
    /// The date when an action was last performed on the file. This includes
    /// actions that might not be immediately obvious, like downloading the file
    /// or marking the file as favorite.
    pub date_recent_action: DateTime<FixedOffset>,
    /// The date when the file's state last changed.
    pub date_state_changed: DateTime<FixedOffset>,
    /// The file's identifier.
    pub id: FileId,
    /// `true` if the file is marked as favorite.
    #[serde(rename = "isFavourite")]
    pub is_favorite: bool,
    /// The file's name.
    pub name: String,
    /// The identifier of the file's parent folder.
    ///
    /// This [`FolderId`](crate::mydoc::FolderId) should only ever be of
    /// variants [`FolderId::Custom`](crate::mydoc::FolderId::Custom) or
    /// [`FolderId::Root`](crate::mydoc::FolderId::Root).
    ///
    /// Trashed files always list
    /// [`FolderId::Root`](crate::mydoc::FolderId::Root) as their parent folder.
    /// To check if a file is trashed, you can query its state instead.
    pub parent_id: FolderId,
    /// The file's state.
    pub state: State,
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
    /// The folder's color.
    pub color: FolderColor,
    /// The date when the folder was last changed.
    pub date_changed: DateTime<FixedOffset>,
    /// The date when the folder was created.
    pub date_created: DateTime<FixedOffset>,
    /// The date when the folder's state last changed.
    pub date_state_changed: DateTime<FixedOffset>,
    /// `true` if the folder has subfolders.
    #[serde(rename = "hasSubFolders")]
    pub has_subfolders: bool,
    /// The folder's identifier.
    pub id: CustomFolderId,
    /// `true` if the folder is marked as favorite.
    #[serde(rename = "isFavourite")]
    pub is_favorite: bool,
    /// The folder's name.
    pub name: String,
    /// The identifier of the folder's parent folder.
    ///
    /// This [`FolderId`](crate::mydoc::FolderId) should only ever be of
    /// variants [`FolderId::Custom`](crate::mydoc::FolderId::Custom) or
    /// [`FolderId::Root`](crate::mydoc::FolderId::Root).
    ///
    /// Trashed folders always list
    /// [`FolderId::Root`](crate::mydoc::FolderId::Root) as their parent folder.
    /// To check if a folder is trashed, you can query its state instead.
    pub parent_id: FolderId,
    /// The folder's state.
    pub state: State,
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
    /// A user-created folder in the virtual file system.
    Custom(CustomFolderId),
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
    /// The date when the recorded event happened.
    pub date: DateTime<FixedOffset>,
    /// `true` if the entry represents a "download event", like viewing the file
    /// or downloading the file.
    pub is_download_event: bool,
    /// `true` if the entry represents a "special event".
    /// TODO: Figure out what this means.
    pub is_special_event: bool,
    /// A textual representation of the recorded event.
    pub text: String,
    /// The user who performed the action.
    pub user: HistoryEntryUser,
}

/// A user who performed an action on a file or folder.
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub struct HistoryEntryUser {
    /// The user's identifier, which seems to equal
    /// `"{school-id}_{user-id}_{account-id}"`.
    #[serde(rename = "userIdentifier")]
    pub id: String,
    /// The user's name.
    pub name: String,
    /// The user's picture hash.
    #[serde(rename = "userPictureHash")]
    pub picture_hash: String,
}

/// A revision of a file in the virtual file system.
// The server response also contains a `location` field which seems to equal
// `{school-id}_{user-id}_{account-id}_{revision-id}`.
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    /// The date when the revision was made.
    #[serde(rename = "dateCreated")]
    pub date: DateTime<FixedOffset>,
    /// The identifier of the associated file.
    pub file_id: FileId,
    /// The name of the associated file.
    #[serde(rename = "label")]
    pub file_name: String,
    /// The size of the associated file in bytes.
    pub file_size: u64,
    /// The revision's identifier.
    pub id: RevisionId,
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
    /// A trashed file or folder.
    Trashed,
    /// A deleted file or folder.
    Deleted,
}

/// A template for a file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Template<'a> {
    /// The default template for documents.
    Word,
    /// The default template for spreadsheets.
    Excel,
    /// The default template for presentations.
    PowerPoint,
    /// A custom school-specific template, identified by a template identifier
    /// string.
    ///
    /// Fetching a template identifier string automatically is not yet possible
    /// and must happen manually for now.
    Custom(&'a str),
}

impl<'a> Template<'a> {
    fn as_str(&self) -> &'static str {
        match self {
            Template::Word => "word",
            Template::Excel => "excel",
            Template::PowerPoint => "powerpoint",
            Template::Custom(_) => "template",
        }
    }
}

#[derive(Deserialize)]
struct Upload {
    pub files: HashMap<String, File>,
}
