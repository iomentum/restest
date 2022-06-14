//! The various states of a request.
//!
//! A request has a specific lifecycle:
//!   - a [`Request`] is created using one of [`Request::get`],
//! [`Request::post`] and so on,
//!   - the request can be modified using the different methods un [`Request`], such as
//! [`with_body`](Request::with_body) or [`with_header`](Request::with_header),
//!   - the [`Request`] is passed as argument of
//! [`Context::run`](crate::Context), returning a [`RequestResult`],
//!   - the final request body is constructed by calling
//! [`expect_status`](RequestResult::expect_status).
//!
//! The documentation for [`Request`] provide more specific description.

use std::collections::HashMap;

use http::status::StatusCode;
use reqwest::Response;
use serde::{de::DeserializeOwned, Serialize};

use crate::url::IntoUrl;

/// An HTTP request we're about to run.
///
/// # Creating a request
///
/// First a [`Request`] must be created. This object will allow to
/// encode the request information. It can be created with [`Request::get`]
/// or [`Request::post`], depending on the kind of request needed.
///
/// Then, various request metadata can be encoded in the builder once it is
/// created. For instance, one can use the
/// [`with_header`](Request::with_header) method to specify a header key
/// to the request.
///
/// Once the metadata is encoded, the [`with_body`](Request::with_body)
/// method allows to specify a body.
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
///     .with_body(GetUsersFilter::All);      // ... Adding a body
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
    pub(crate) context_description: String,
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
    pub fn get(url: impl IntoUrl) -> Request<()> {
        let url = url.into_url();
        Request {
            body: (),
            header: HashMap::new(),
            method: Method::Get,
            context_description: format!("GET:{}", url),
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
    pub fn post(url: impl IntoUrl) -> Request<()> {
        let url = url.into_url();
        Request {
            body: (),
            header: HashMap::new(),
            method: Method::Post,
            context_description: format!("POST:{}", url),
            url,
        }
    }

    /// Creates a PUT request builder for a specific URL.
    ///
    /// # Specifying an URL
    ///
    /// The url argument must be either a string literal or the value produced
    /// by the [`path`] macro. Only the absolute path to the resource must be
    /// passed.
    ///
    /// Refer to the [`get`][Request::get] method documentation for a
    /// self-describing example.
    pub fn put(url: impl IntoUrl) -> Request<()> {
        let url = url.into_url();
        Request {
            body: (),
            header: HashMap::new(),
            method: Method::Put,
            context_description: format!("PUT:{}", url),
            url,
        }
    }

    /// Creates a DELETE request builder for a specific URL.
    ///
    /// # Specifying an URL
    ///
    /// The url argument must be either a string literal or the value produced
    /// by the [`path`] macro. Only the absolute path to the resource must be
    /// passed.
    ///
    /// Refer to the [`get`][Request::get] method documentation for a
    /// self-describing example,.
    pub fn delete(url: impl IntoUrl) -> Request<()> {
        let url = url.into_url();
        Request {
            body: (),
            header: HashMap::new(),
            method: Method::Delete,
            context_description: format!("DELETE:{}", url),
            url,
        }
    }
}

/// Allows encode metadata in order to create a [`Request`].
///
/// This type can be created by calling either [`Request::get`],
/// [`Request::post`], [`Request::put`] or [`Request::delete`].
/// Specifically, this type allows to encode the request
/// header with [`Request::with_header`], and to encode the
/// request body with [`Request::with_body`].
///
/// This allows to create [`Request`] types, as shown in the following example:
///
/// ```rust
/// use restest::Request;
///
/// use serde::Serialize;
///
/// let request = Request::get("user")
///     .with_header("token", "mom-said-yes")
///     .with_body(GetUserRequest {
///         login: String::from("jdoe")
///     });
///
/// #[derive(Serialize)]
/// struct GetUserRequest {
///     login: String,
/// }
/// ```
impl<B> Request<B>
where
    B: Serialize,
{
    /// Adds a header key and value to the request.
    pub fn with_header(mut self, key: impl ToString, value: impl ToString) -> Request<B> {
        let previous_entry = self.header.insert(key.to_string(), value.to_string());
        assert!(previous_entry.is_none(), "Attempt to replace a header");

        self
    }

    /// Specifies a body, returns the final [`Request`] object.
    pub fn with_body<C>(self, body: C) -> Request<C>
    where
        C: Serialize,
    {
        let Request {
            header,
            method,
            url,
            context_description,
            ..
        } = self;

        Request {
            body,
            header,
            method,
            url,
            context_description,
        }
    }

    /// Specifies a context description. Returns the final [`Request`] object.
    pub fn with_context(mut self, context_description: impl ToString) -> Request<B> {
        self.context_description = context_description.to_string();

        self
    }
}

impl<B> AsRef<Request<B>> for Request<B>
where
    B: Serialize,
{
    fn as_ref(&self) -> &Request<B> {
        self
    }
}

impl<B> Clone for Request<B>
where
    B: Serialize + Clone,
{
    fn clone(&self) -> Request<B> {
        Request {
            body: self.body.clone(),
            header: self.header.clone(),
            method: self.method,
            url: self.url.clone(),
            context_description: self.context_description.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Method {
    Get,
    Post,
    Put,
    Delete,
}

/// The data returned by the server once the request is performed.
///
/// This datatype is meant for intermediary representation. It can be converted
/// to a concrete type by calling [`RequestResult::expect_status`].
pub struct RequestResult {
    pub(crate) response: Response,
    pub(crate) context_description: String,
}

impl RequestResult {
    /// Checks if the response status meets an expected status code and convert
    /// the body to a concrete type.
    ///
    /// This method uses `serde` internally, so the output type must implement
    /// [`DeserializeOwned`].
    ///
    /// # Panics
    ///
    /// This method panics if the server response status is not equal to
    /// `status` or if the body can not be deserialized to the specified type.
    #[track_caller]
    pub async fn expect_status<T>(self, status: StatusCode) -> T
    where
        T: DeserializeOwned,
    {
        assert_eq!(
            self.response.status(),
            status,
            "Unexpected server response code for request '{}'. Body is {}",
            self.context_description,
            self.response.text().await.unwrap_or_else(|_| panic!(
                "Unexpected server response code for request {}. Unable to read response body",
                self.context_description
            ))
        );

        match self.response.json().await {
            Err(err) => panic!(
                "Failed to deserialize body for request '{}': {}",
                self.context_description, err
            ),
            Ok(res) => res,
        }
    }
}
