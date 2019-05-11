//! # smartschool
//!
//! Smartschool client library for Rust.
//!
//! ## Example
//!
//! A quick usage example using [Runtime](https://crates.io/crates/runtime):
//!
//! ```
//! use smartschool::{mydoc, Client};
//! use std::fs::File;
//!
//! #[runtime::main(runtime_tokio::Tokio)]
//! async fn main() {
//!     let client = await!(Client::login(
//!         "https://myschool.smartschool.be",
//!         "username",
//!         "password"
//!     ))
//!     .expect("error while logging in");
//! }
//! ```

#![feature(async_await, await_macro, custom_attribute)]
#![deny(missing_docs)]

pub mod client;
pub mod error;

pub use client::Client;
