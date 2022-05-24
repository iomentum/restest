//! Hold information about the backend we're about to query.
//!
//! This module provides the [`Context`] type, whose goal is to store
//! information about the backend (its URL base, its port) and to run a
//! [`Request`].

use http::{header::HeaderName, HeaderMap, HeaderValue};
use reqwest::Client;
use serde::Serialize;

use crate::request::{Method, Request, RequestResult};

/// A structure that holds information about the backend we're about to query.
///
/// All its setters are `const`, meaning it can be placed in a module, and
/// accessed from anywhere in the module.
///
/// # Example
///
/// ```rust
/// use restest::{Context, Request};
///
/// const CONTEXT: Context = Context::new()
///     .with_port(80)
///     .with_host("http://localhost");
///
/// #[tokio::test]
/// async fn first_test() {
///     // Use CONTEXT.run(...) to run a request.
/// }
///
/// #[tokio::test]
/// async fn second_test() {
///     // Use CONTEXT.run(...) to run another request.
/// }
/// ```
pub struct Context {
    host: &'static str,
    port: u16,
}

impl Context {
    /// Creates a new context with default values.
    ///
    /// The default host is localhost.
    ///
    /// The default port is port `80`.
    pub const fn new() -> Context {
        Context {
            host: "http://localhost",
            port: 80,
        }
    }

    /// Sets a host value.
    ///
    /// The previously-set host is discarded.
    pub const fn with_host(self, host: &'static str) -> Context {
        let port = self.port;

        Context { host, port }
    }

    /// Sets a port value.
    ///
    /// The previously-set port is discarded.
    pub const fn with_port(self, port: u16) -> Context {
        let host = self.host;

        Context { host, port }
    }

    /// Runs a request.
    ///
    /// This function performs I/O, therefore it is marked as `async`.
    pub async fn run<I>(&self, request: Request<I>) -> RequestResult
    where
        I: Serialize,
    {
        let client = reqwest::Client::new();

        let create_request = match request.method {
            Method::Get => Client::get,
            Method::Post => Client::post,
            Method::Put => Client::put,
            Method::Delete => Client::delete,
        };

        let url = format!("{}:{}{}", self.host, self.port, request.url);

        let headers = request
            .header
            .into_iter()
            .map(|(k, v)| {
                (
                    k.parse::<HeaderName>()
                        .expect("Header name conversion failed"),
                    v.parse::<HeaderValue>()
                        .expect("Header value conversion failed"),
                )
            })
            .collect::<HeaderMap<HeaderValue>>();

        let response = create_request(&client, url)
            .headers(headers)
            .json(&request.body)
            .send()
            .await
            .expect("Request failed");

        RequestResult { response }
    }
}
