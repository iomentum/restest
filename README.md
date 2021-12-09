# restest

Black-box integration test for REST APIs in Rust.

This crate provides the [`assert_api`] macro that allows to declaratively
test, given a certain request, that the response emitted by the server is
correct.

## Example

```rust
#![feature(assert_matches)]

use serde::{Deserialize, Serialize};

restest::port! { 8080 }

restest::assert_api! {
    POST "/user",
    PostUser {
        year_of_birth: 2000,
    } => User {
        year_of_birth: 2000,
        ..
    }
}

#[derive(Debug, Serialize)]
struct PostUser {
    year_of_birth: usize,
}

#[derive(Debug, Deserialize)]
struct User {
    year_of_birth: usize,
    id: Uuid
}
```

## Writing tests

The [`port`] macro sets the port at which the request must be run.

The tests are written as normal Rust tests (ie: functions annotated with
`#[test]`). As we're using asynchronous code, we must write async tests,
perhaps using `#[tokio::test]`.

More specifically, the [`assert_api`] macro can be used in order to query
the server API and analyze its response.

## Running tests

The server must be running in the background when `cargo test` is run.

## Required toolchain

The `nightly` feature allows the [`assert_api`] macro to expand to
nightly-specific code. This offers the following features:

- better panic message when the request body does not match the expected
  pattern (requires [`assert_matches`]),
- ability to reuse variables matched in the response body (requires
  [`let_else`]) (_still WIP_).

These two features must be added at the crate root using the following two
lines:

```rust
#![feature(assert_matches)]
#![feature(let_else)]
```

[`assert_matches`]: https://github.com/rust-lang/rust/issues/82775
[`let_else`]: https://github.com/rust-lang/rust/issues/87335

## Code of Conduct

We have a Code of Conduct so as to create a more enjoyable community and
work environment. Please see the [CODE_OF_CONDUCT](CODE_OF_CONDUCT.md)
file for more details.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Dual MIT/Apache2 is strictly more permissive
