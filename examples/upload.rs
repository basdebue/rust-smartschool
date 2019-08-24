use smartschool::{
    error::Result,
    mydoc::{self, FolderId},
    upload::{self, File},
    Client,
};
use std::fs;

/// Uploads a local file to the root folder.
#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;

    let bytes = fs::read("example.txt").unwrap(); // you should probably handle this error
    let file = File::from_bytes(bytes).build("uploaded_example.txt");

    let upload_dir = upload::get_upload_directory(&client).await?;
    upload::upload_file(&client, upload_dir.clone(), file).await?;
    mydoc::upload(&client, FolderId::Root, &upload_dir).await?;

    Ok(())
}
