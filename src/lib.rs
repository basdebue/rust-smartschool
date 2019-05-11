//! # smartschool
//!
//! Smartschool client library for Rust.
//!
//! ## Example
//!
//! A quick usage example using [Runtime](https://crates.io/crates/runtime):
//!
//! ```rust
//! #![feature(async_await)]
//!
//! use smartschool::error::Result;
//! use smartschool::Client;
//!
//! #[runtime::main(runtime_tokio::Tokio)]
//! async fn main() -> Result<()> {
//!     let _client = Client::login(
//!         "https://myschool.smartschool.be",
//!         "username",
//!         "password"
//!     ).await?;
//!     Ok(())
//! }
//! ```

#![feature(async_await, custom_attribute)]
#![deny(missing_docs)]

pub mod client;
pub mod error;

pub use client::Client;
