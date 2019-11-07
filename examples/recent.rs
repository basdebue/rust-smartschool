use smartschool::{error::Result, mydoc, Client};

/// Prints a list of recently modified files.
#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;

    let files = mydoc::get_recent_files(&client).await?;
    if !files.is_empty() {
        for file in files {
            println!("{}", file.name);
        }
    } else {
        println!("No recently modified files...");
    }

    Ok(())
}
