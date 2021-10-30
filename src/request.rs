//! An HTTP request we're about to run.

use std::collections::HashMap;

use http::status::StatusCode;
use reqwest::Response;
use serde::{de::DeserializeOwned, Serialize};

use crate::url::IntoUrl;

/// An HTTP request we're about to run.
///
/// # Creating a request
///
/// First a [`RequestBuilder`] must be created. This object will allow to
/// encode the request information. It can be created with [`Request::get`]
/// or [`Request::post`], depending on the kind of request needed.
///
/// Then, various request metadata can be encoded in the builder once it is
/// created. For instance, one can use the
/// [`with_header`](RequestBuilder::with_header) method to specify a header key
/// to the request.
///
/// After every metadata is encoded, the
/// [`with_body`](RequestBuilder::with_body) method allows to specify a body and
/// create the final [`Request`] object.
///
/// The following code snippet shows all these three steps:
///
/// ```rust
/// use restest::Request;
///
/// use serde::Serialize;
///
/// let request = Request::get("users")       // Creating the builder...
///     .with_header("token", "mom-said-yes") // ... Adding metadata to the builder
///     .with_body(GetUsersFilter::All);      // ... Adding a body, building the final Request.
///
/// #[derive(Serialize)]
/// enum GetUsersFilter {
///     All,
///     YoungerThan(u8),
///     OlderThan(u8),
/// }
/// ```
///
/// # Running a request
///
/// Once the [`Request`] has been successfully created, it can be run by using
/// the [`Context::run`](crate::Context::run) method.
pub struct Request<B>
where
    B: Serialize,
{
    pub(crate) body: B,
    pub(crate) header: HashMap<String, String>,
    pub(crate) method: Method,
    pub(crate) url: String,
}

impl Request<()> {
    /// Creates a GET request builder for a specific URL.
    ///
    /// # Specifying an URL
    ///
    /// The url argument must be either a string literal or the value produced
    /// by the [`path`] macro. Only the absolute path to the resource must be
    /// passed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use restest::{path, Request};
    ///
    /// let request_1 = Request::get("users");
    ///
    /// let user_name = "scrabsha";
    /// let request_2 = Request::get(path!["users", user_name]);
    /// ```
    pub fn get(url: impl IntoUrl) -> RequestBuilder {
        let url = url.into_url();
        RequestBuilder {
            header: HashMap::new(),
            method: Method::Get,
            url,
        }
    }

    /// Creates a POST request builder for a specific URL.
    ///
    /// # Specifying an URL
    ///
    /// The url argument must be either a string literal or the value produced
    /// by the [`path`] macro. Only the absolute path to the resource must be
    /// passed.
    ///
    /// Refer to the [`get`][Request::get] method documentation for a
    /// self-describing example.
    pub fn post(url: impl IntoUrl) -> RequestBuilder {
        let url = url.into_url();
        RequestBuilder {
            header: HashMap::new(),
            method: Method::Post,
            url,
        }
    }
}

pub struct RequestBuilder {
    header: HashMap<String, String>,
    method: Method,
    url: String,
}

impl RequestBuilder {
    pub fn with_header(mut self, key: impl ToString, value: impl ToString) -> RequestBuilder {
        let previous_entry = self.header.insert(key.to_string(), value.to_string());
        assert!(previous_entry.is_none(), "Attempt to replace a header");

        self
    }

    pub fn with_body<B>(self, body: B) -> Request<B>
    where
        B: Serialize,
    {
        let RequestBuilder {
            header,
            method,
            url,
        } = self;

        Request {
            body,
            header,
            method,
            url,
        }
    }
}

pub(crate) enum Method {
    Get,
    Post,
}

pub struct RequestResult {
    pub(crate) response: Response,
}

impl RequestResult {
    #[track_caller]
    pub async fn expect_status<T>(self, status: StatusCode) -> T
    where
        T: DeserializeOwned,
    {
        assert_eq!(self.response.status(), status);

        self.response
            .json()
            .await
            .expect("Failed to deserialize body")
    }
}
