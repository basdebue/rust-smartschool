# smartschool

[![crates.io](https://img.shields.io/crates/v/smartschool.svg)](https://crates.io/crates/smartschool)
[![matrix](https://img.shields.io/matrix/rust-smartschool:matrix.org.svg)](https://matrix.to/#/#rust-smartschool:matrix.org)

Smartschool client library for Rust.

## Example

A quick usage example using [Runtime](https://crates.io/crates/runtime):

```rust
#[feature(async_await)]

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
for inclusion in `smartschool` by you, shall be licensed as MIT, without any additional
terms or conditions.