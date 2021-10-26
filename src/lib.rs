//! Black-box integration test for REST APIs in Rust.
//!
//! This crate provides the [`assert_api`] macro that allows to declaratively
//! test, given a certain request, that the response emitted by the server is
//! correct.
//!
//! # Example
//!
//! ```no_run
//! /*
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
//! */
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

/// Asserts that a response body matches a given pattern, adds
/// bindings to the current scope.
///
/// This has *very* limited functionalities for now.
///
/// # Example
///
/// ```rust
/// restest::assert_body_matches!(
///     [42, 41],
///     [42, 41]
/// );
///
/// restest::assert_body_matches! {
///     [42, 101],
///     a
/// };
///
/// assert_eq!(a, [42, 101]);
/// ```
pub use restest_macros::assert_body_matches;

pub mod context;
pub mod request;
pub mod url;

pub use context::Context;
pub use request::Request;
pub use url::IntoUrl;

/// Creates a path from multiple segments.
///
/// All the segments don't need to have the same type. They all need to
/// implement [`ToString`].
///
/// # Example
///
/// ```rust
/// use restest::{path, Request};
///
/// let my_user_id = Uuid::new_v4();
///
/// Request::get(path!["users", my_user_id])
///     .with_body(())
///     // the rest of the request
/// #   ;
/// # struct Uuid;
/// # impl Uuid {
/// #     fn new_v4() -> usize { 42 }
/// # }
/// ```
#[macro_export]
macro_rules! path {
    ( $( $segment:expr ),* $(,)? ) => {
        vec![ $( Box::new($segment) as Box<dyn ToString>, )* ]
    };
}
