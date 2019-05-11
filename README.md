# smartschool

[![crates.io](https://img.shields.io/crates/v/smartschool.svg)](https://crates.io/crates/smartschool)
[![matrix](https://img.shields.io/matrix/rust-smartschool:matrix.org.svg)](https://matrix.to/#/#rust-smartschool:matrix.org)

Smartschool client library for Rust.

## Example

A quick usage example using [Runtime](https://crates.io/crates/runtime):

```rust
#![feature(async_await)]

use smartschool::error::Result,
use smartschool::Client;

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<()> {
    let _client = Client::login(
        "https://myschool.smartschool.be",
        "username",
        "password"
    ).await?;
    Ok(())
}
```

## Contributing

Thank you for your interest in contributing to this project! Please check out our [contributing guide](CONTRIBUTING.md) to get started.

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `rust-smartschool` by you, shall be licensed as MIT, without any additional
terms or conditions.

## Legal

Using this crate may or may not be against [Smartschool's end-user license agreement](https://www.smartschool.be/gebruikersovereenkomst/). Section 4 of the EULA states that it is prohibited to use "software intended for data collection", such as "spiders, crawlers, keyloggers, robots and similar software". It is up to you to determine whether a program using this crate qualifies as such software.

Disclaimer: The name "Smartschool" is a copyright of Smartbit bvba. This project is in no way affiliated with or endorsed by Smartbit bvba. The developers of this project are not responsible for any legalities that may arise in the use of this project.