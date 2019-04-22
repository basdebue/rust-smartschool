//! smartschool

#![feature(async_await, await_macro, custom_attribute, futures_api)]

pub mod client;
pub mod error;

pub use client::Client;
