//! Smartschool client library for Rust.
//!
//! This crate is structured according to Smartschool's internal API structure.
//! Generally, API modules should map to Rust modules and API methods should map
//! to free-standing asynchronous functions.
//!
//! ## Example
//!
//! A simple usage example:
//!
//! ```ignore
//! use smartschool::{error::Result, mydoc, Client};
//!
//! /// Prints a list of recently modified files.
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;
//!
//!     let files = mydoc::get_recent_files(&client).await?;
//!     if !files.is_empty() {
//!        for file in files {
//!             println!("{}", file.name);
//!         }
//!     } else {
//!         println!("No recently modified files...");
//!     }
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

pub use client::Client;
pub use error::Error;

pub mod client;
pub mod error;
pub mod mydoc;
mod serde;
pub mod upload;
