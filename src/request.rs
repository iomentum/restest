use std::collections::HashMap;

use http::status::StatusCode;
use reqwest::Response;
use serde::{de::DeserializeOwned, Serialize};

use crate::url::IntoUrl;

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
    pub fn get(url: impl IntoUrl) -> RequestBuilder {
        let url = url.into_url();
        RequestBuilder {
            header: HashMap::new(),
            method: Method::Get,
            url,
        }
    }

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
    pub fn expect_status(self, status: StatusCode) -> Self {
        assert_eq!(self.response.status(), status);
        self
    }

    #[track_caller]
    pub async fn deserialize<T: DeserializeOwned>(self) -> T {
        self.response
            .json()
            .await
            .expect("Failed to deserialize body")
    }
}
