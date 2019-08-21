# smartschool

[![crates.io](https://img.shields.io/crates/v/smartschool.svg)](https://crates.io/crates/smartschool)
[![API docs](https://docs.rs/smartschool/badge.svg)](https://docs.rs/smartschool)

A Smartschool client library for Rust.

## Example

A simple usage example:

```rust
#![feature(async_await)]

use smartschool::{error::Result, mydoc, Client};

/// Prints a list of recently modified files.
#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::login("https://myschool.smartschool.be", "username", "password").await?;

    let files = mydoc::get_recent_files(&client).await?;
    if !files.is_empty() {
        for file in files {
            println!("{}", file.name());
        }
    } else {
        println!("No recently modified files...");
    }

    Ok(())
}
```

## Scope

This project aims to cover all JSON-based modules of Smartschool's API. XML-based modules, such as Messages, do **not** fall within the scope of the project, since an idiomatic XML abstraction for Rust has yet to be found and Smartschool seems to be in the process of transitioning all of its modules to JSON anyway.

We are not explicitly targeting Smartschool's internal API; any contributions relating to its public API are also welcome!

## Contributing

Thank you for your interest in contributing to this project! Please check out our [contributing guide](CONTRIBUTING.md) to get started.

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `rust-smartschool` by you, shall be licensed as MIT, without any additional terms or conditions.

## Legal

Using this crate may or may not violate [Smartschool's end-user license agreement](https://www.smartschool.be/gebruikersovereenkomst/). Section 4 of the EULA states that it is prohibited to use "software intended for data collection", such as "spiders, crawlers, keyloggers, robots and similar software". It is up to you to determine whether a program using this crate qualifies as such software.

Disclaimer: The name "Smartschool" is a copyright of Smartbit bvba. This project is in no way affiliated with or endorsed by Smartbit bvba. The developers of this project are not responsible for any legalities that may arise in the use of this project.