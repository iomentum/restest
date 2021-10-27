//! Black-box integration test for REST APIs in Rust.
//!
//! `restest` provides primitives that allow to write REST API in a declarative
//! manner.
//!
//! # Example
//!
//! ```no_run
//! use restest::{assert_body_matches, path, Context, Request};
//!
//! use serde::{Deserialize, Serialize};
//! use http::StatusCode;
//!
//! const CONTEXT: Context = Context::new().with_port(8080);
//!
//! # #[tokio::main]
//! # async fn main() {
//! let request = Request::post(path!["users"]).with_body(PostUser {
//!     year_of_birth: 2000,
//! });
//!
//! let body = CONTEXT
//!     .run(request)
//!     .await
//!     .expect_status(StatusCode::OK)
//!     .await;
//!
//! assert_body_matches! {
//!     body,
//!     User {
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
//! Writing tests using `restest` always follow the same patterns. They are
//! described below.
//!
//! ## Specifying the context
//!
//! The [`Context`] object handles all the server-specific configuration:
//!   - which base URL should be used,
//!   - which port should be used.
//!
//! It can be created with [`Context::new`]. All its setters are `const`, so
//! it can be initialized once for all the tests of a module:
//!
//! ```rust
//! use restest::Context;
//!
//! const CONTEXT: Context = Context::new().with_port(8080);
//!
//! #[tokio::test]
//! async fn test_first_route() {
//!     // Test code that use `CONTEXT` for a specific route
//! }
//!
//! #[tokio::test]
//! async fn test_second_route() {
//!     // Test code that use `CONTEXT` again for another route
//! }
//! ```
//!
//! As we're running `async` code under the hood, all the tests must be `async`,
//! hence the use of `tokio::test`
//!
//! # Creating a request
//!
//! Let's focus on the test function itself.
//!
//! The first thing to do is to create a [`Request`] object. This object allows
//! to specify characteristics about a specific request that is performed later.
//!
//! Running [`Request::get`] allows to construct a GET request to a specific
//! URL. Header keys can be specified by calling the
//! [`with_header`](request::RequestBuilder::with_header) method. The
//! final [`Request`] is built by calling the
//! [`with_body`](request::RequestBuilder::with_body) method, which
//! allows to add a body.
//!
//! ```rust
//! use restest::{path, Request};
//!
//! let request = Request::get(path!["users", "scrabsha"])
//!     .with_header("token", "mom-said-yes")
//!     .with_body(());
//! ```
//!
//! Similarly, POST requests can be creating by using [`Request::post`] instead
//! of [`Request::get`].
//!
//! # Performing the request
//!
//! Once that the [`Request`] object has been created, we can run the request
//! by passing the [`Request`] to the [`Context`] when calling [`Context::run`].
//! Once `await`-ed, the [`expect_status`](request::RequestResult::expect_status) method
//! checks for the request status code and converts the response body to the
//! expected output type.
//!
//! ```rust,no_run
//! use http::StatusCode;
//! use uuid::Uuid;
//! use serde::Deserialize;
//!
//! # use restest::{Context, Request};
//! # const CONTEXT: Context = Context::new();
//! # #[tokio::main]
//! # async fn main() {
//! # let request = Request::get("foo").with_body(());
//! let user: User = CONTEXT
//!     .run(request)
//!     .await
//!     .expect_status(StatusCode::OK)
//!     .await;
//! # }
//!
//! #[derive(Deserialize)]
//! struct User {
//!     name: String,
//!     age: u8,
//!     id: Uuid,
//! }
//! ```
//!
//! # Checking the response body
//!
//! Properties about the response body can be asserted with
//! [`assert_body_matches`]. The macro supports the full rust pattern syntax,
//! making it easy to check for expected values and variants. It also provides
//! bindings, allowing you to bring data from the body in scope:
//!
//! ```rust
//! use restest::assert_body_matches;
//! use uuid::Uuid;
//!
//! # let user = User {
//! #     name: "Grace Hopper".to_string(),
//! #     age: 85,
//! #     id: Uuid::new_v4(),
//! # };
//! #
//! assert_body_matches! {
//!     user,
//!     User {
//!         name: "Grace Hopper",
//!         age: 85,
//!         id,
//!     },
//! }
//!
//! // id is now a variable that can be used:
//! println!("Grace Hopper has id `{}`", id);
//! #
//! # #[derive(serde::Deserialize)]
//! # struct User {
//! #     name: String,
//! #     age: u8,
//! #     id: Uuid,
//! # }
//! ```
//!
//! The extracted variable can be used for next requests or more complex
//! testing.
//!
//! *And that's it!*

/// Asserts that a response body matches a given pattern, adds
/// bindings to the current scope.
///
/// This pattern supports all the Rust pattern syntax, with a few additions:
///   - matching on `String` can be done with string literals,
///   - values that are bound to variables are available in the whole scope,
///     allowing for later use.
///
/// # Example
///
/// The following code demonstrate basic matching:
///
/// ```rust,no_run
/// use restest::assert_body_matches;
///
/// struct User {
///     name: String,
///     age: u8,
/// }
///
/// let user = get_user();
///
/// assert_body_matches! {
///     user,
///     User {
///         name: "John Doe",
///         age: 48,
///     },
/// }
///
/// fn get_user() -> User {
///     /* Obscure code */
/// # User {
/// #     name: "John Doe".to_string(),
/// #     age: 48,
/// # }
/// }
/// ```
///
/// Values can be brought to scope:
///
/// ```rust
/// use restest::assert_body_matches;
/// use uuid::Uuid;
///
/// struct User {
///     id: Uuid,
///     name: String,
/// }
///
/// let user = get_user();
///
/// assert_body_matches! {
///     user,
///     User {
///         id,
///         name: "John Doe",
///     },
/// }
///
/// // id is now available:
/// println!("John Doe has id `{}`", id);
///
/// fn get_user() -> User {
///     /* Obscure code */
/// #    User {
/// #        id: Uuid::new_v4(),
/// #        name: "John Doe".to_string(),
/// #    }
/// }
/// ```
///
/// Bringing values to scope may allow to extract information that are required
/// to perform a next request.
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
