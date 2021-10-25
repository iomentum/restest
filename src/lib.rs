//! Black-box integration test for REST APIs in Rust.
//!
//! This crate provides the [`assert_api`] macro that allows to declaratively
//! test, given a certain request, that the response emitted by the server is
//! correct.
//!
//! # Example
//!
//! ```no_run
//! #![feature(assert_matches)]
//! #![feature(let_else)]
//!
//! use serde::{Deserialize, Serialize};
//!
//! restest::port! { 8080 }
//!
//! # #[tokio::main]
//! # async fn main() {
//! restest::assert_api! {
//!     POST "/user",
//!     PostUser {
//!         year_of_birth: 2000,
//!     } => User {
//!         year_of_birth: 2000,
//!         ..
//!     }
//! }
//! # }
//!
//! #[derive(Debug, Serialize)]
//! struct PostUser {
//!     year_of_birth: usize,
//! }
//!
//! #[derive(Debug, Deserialize)]
//! struct User {
//!     year_of_birth: usize,
//!     id: Uuid
//! }
//! # #[derive(Debug, Deserialize)]
//! # struct Uuid;
//! ```
//!
//! # Writing tests
//!
//! The [`port`] macro sets the port at which the request must be run.
//!
//! The tests are written as normal Rust tests (ie: functions annotated with
//! `#[test]`). As we're using asynchronous code, we must write async tests,
//! perhaps using `#[tokio::test]`.
//!
//! More specifically, the [`assert_api`] macro can be used in order to query
//! the server API and analyze its response.
//!
//! # Running tests
//!
//! The server must be running in the background when `cargo test` is run.
//!
//! # Required toolchain
//!
//! The `nightly` feature allows the [`assert_api`] macro to expand to
//! nightly-specific code. This offers the following features:
//!   - better panic message when the request body does not match the expected
//!   pattern (requires [`assert_matches`]),
//!   - ability to reuse variables matched in the response body (requires
//!   [`let_else`]) (*still WIP*).
//!
//! These two features must be added at the crate root using the following two
//! lines:
//!
//! ```
//! #![feature(assert_matches)]
//! #![feature(let_else)]
//! ```
//!
//! [`assert_matches`]: https://github.com/rust-lang/rust/issues/82775
//! [`let_else`]: https://github.com/rust-lang/rust/issues/87335

/// Asserts that a [`Value`][serde_json::Value] matches a given pattern, adds
/// bindings to the current scope.
///
/// This has *very* limited functionalities for now.
///
/// # Example
///
/// ```rust
/// restest::assert_body_matches!(
///     serde_json::json! {
///         [42, 41]
///     },
///     [42, 41]
/// );
///
/// restest::assert_body_matches! {
///     serde_json::json! {
///         [42, 101]
///     },
///     a as [isize]
/// };
///
/// assert_eq!(a, [42, 101]);
/// ```
pub use restest_macros::assert_body_matches;

#[doc(hidden)]
pub mod __private;
pub mod context;
pub mod request;
pub mod url;

pub use context::Context;

use std::fmt::Display;

use reqwest::{Client, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};

// Important note:
// All the examples and tests in this crate must be run **with** the `nightly`
// feature and using the nightly toolchain.

#[macro_export]
macro_rules! path {
    ( $( $segment:expr ),* $(,)? ) => {
        vec![ $( Box::new($segment) as Box<dyn ToString>, )* ]
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! method {
    ( GET ) => {
        $crate::Method::Get
    };
    ( POST ) => {
        $crate::Method::Post
    };
}

/// The whole point.
///
/// This macro sends a request to a REST endpoint and ensures that the server
/// response matches a given pattern.
///
/// # Example
///
/// Below is a simple example to test what an hypothetic server returns when we
/// add a new user:
///
/// ```no_run
/// #![feature(assert_matches)]
/// #![feature(let_else)]
///
/// use serde::{Deserialize, Serialize};
/// # #[tokio::main]
/// # async fn main() {
///
/// restest::port! { 8080 };
///
/// restest::assert_api! {
///     POST "/user",
///     PostUser {
///         year_of_birth: 2000,
///     } => User {
///         year_of_birth: 2000,
///         ..
///     }
/// }
/// # }
///
/// #[derive(Debug, Serialize)]
/// struct PostUser {
///     year_of_birth: usize,
/// }
///
/// #[derive(Debug, Deserialize)]
/// struct User {
///     year_of_birth: usize,
///     id: Uuid
/// }
/// #
/// # #[derive(Debug, Deserialize)]
/// # struct Uuid;
/// ```
///
/// # Syntax
///
/// The following snippet describes the macro syntax:
///
/// ```none
/// assert_api! {
///     <method> <url>,
///     <input> => <output>
/// }
/// ```
///
/// ## Specifying the method and url
///
/// `method` is one of `GET` or `POST`.
///
/// `url` is currently a string literal, this will change in the future. We may
/// allow multiple path segments of different types to be concatenated at
/// runtime.
///
/// ## Specifying the request input and output body
///
/// `input` must be an expression of any type that implements [`Serialize`][serde::Serialize].
///
/// `output` must be a pattern of any type that implements [`Deserialize`][serde::Deserialize].
/// We use a pattern here because it allows us to not check values that are
/// randomly generated by the server, such as ids. This also allows us to
/// expect a specific enum variant.
///
/// ## Reusing variables from the response body
///
/// **Note**: *this feature requires the nightly toolchain, `restest`must be
/// compiled with the `nightly` feature enabled.*
///
/// Variables matched in the output pattern can be reused in the code following
/// the call to [`assert_api`]:
///
/// ```no_run
/// #![feature(assert_matches)]
/// #![feature(let_else)]
///
/// use serde::Deserialize;
///
/// # #[tokio::main]
/// # async fn main() {
/// restest::port! { 8080 };
///
/// restest::assert_api! {
///     GET "/users/Lovelace",
///     () => User {
///         id,
///         ..
///     }
/// }
///
/// println!("Ada Lovelace has id `{:?}`", id);
///
/// #[derive(Debug, Deserialize)]
/// struct User {
///     id: Uuid,
///     first_name: String,
///     last_name: String,
/// }
/// # #[derive(Debug, Deserialize)]
/// # struct Uuid;
/// # }
/// ```
///
/// # Server address
///
/// We always perform HTTP requests on localhost for now. That may be changed
/// in the future, but is considered *good enough* for a MVP.
///
/// # Panics
///
/// This macro will panic in one of the following case:
///   - it fails to send an HTTP request to the specified URL,
///   - it fails do convert the response body to the output type,
///   - the server the server output does not match the expected output pattern.
#[macro_export]
macro_rules! assert_api {
    // Future work checklist:
    //
    // This syntax does not allow to specify a header.
    //
    // This syntax does not allow to assert a response status code.
    (
        $method:ident $url:expr,
        $input:expr => $output:pat $(,)?
    ) => {
        let server_output = $crate::request_and_deserialize(
            $crate::method!($method),
            $url,
            __RESTEST_INTERNAL__PORT,
            &$input,
        )
        .await;

        $crate::match_and_maybe_bind!($output, server_output);
    };
}

// There's a bug in rustfmt where things that come next to a #[macro] in a
// declarative macro are shifted four spaces to the right at each format.
//
// As a fix, the said code is #[rustfmt::skip]ped place.
#[doc(hidden)]
#[cfg(feature = "nightly")]
#[rustfmt::skip]
#[macro_export]
macro_rules! match_and_maybe_bind {
    ( $pat:pat, $expr:expr ) => {
        #[allow(unused_variables)]
        {
            std::assert_matches::assert_matches!(&$expr, $pat);
        }

        let $pat = $expr else { unreachable!() };
    };
}

#[doc(hidden)]
#[cfg(not(feature = "nightly"))]
#[macro_export]
macro_rules! match_and_maybe_bind {
    ( $pat:pat, $expr:expr ) => {
        assert!(matches!($output, $expr));
    };
}

/// Sets which port to use for the tests.
///
/// If it is called in a module, then it sets the port for the whole module. If
/// called in a function, then it sets the port for the whole function. It
/// accepts only expressions that can be evaluated as `const` and whose return
/// type is [`u16`].
///
/// # Example
///
/// The following code sets port 8080 as a testing port for the whole module:
///
/// ```
/// restest::port! { 8080 }
/// ```
#[macro_export]
macro_rules! port {
    ($port:expr) => {
        #[doc(hidden)]
        #[allow(dead_code)]
        const __RESTEST_INTERNAL__PORT: u16 = $port;
    };
}

const LOCALHOST_ADDRESS: &str = "http://127.0.0.1";

#[doc(hidden)]
pub enum Method {
    Get,
    Post,
}

#[doc(hidden)]
#[track_caller]
// As we're writing test code, we can panic! at our own pace. The use of
// #[track_caller] will report the correct span to the user anyway.
pub async fn request_and_deserialize<I, O, U>(method: Method, url: U, port: u16, input: &I) -> O
where
    I: Serialize,
    O: DeserializeOwned,
    U: Display,
{
    let url = create_request_url(port, url);

    create_request_builder(method, url)
        .json(input)
        .send()
        .await
        .expect("Failed to perform HTTP request")
        .json()
        .await
        .expect("Failed to convert response body")
}

fn create_request_url(port: u16, url: impl Display) -> String {
    format!("{}:{}{}", LOCALHOST_ADDRESS, port, url)
}

fn create_request_builder(method: Method, url: String) -> RequestBuilder {
    let client = Client::new();
    let create_request = match method {
        Method::Get => Client::get,
        Method::Post => Client::post,
    };
    create_request(&client, url)
}
