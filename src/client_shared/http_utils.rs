use std::marker::PhantomData;

#[cfg(feature = "async-compat")]
use async_compat::Compat;
use reqwest::{RequestBuilder, Response};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

// use super::errors::{registration_error, RegistrationError};


// Serde objects

/// Serde struct used to deserialize the
/// json body of a poll response
#[derive(Deserialize)]
pub(crate) struct PollResponse {
    pub(crate) aes_key: String,

    #[serde(rename(deserialize = "data"))]
    pub(crate) data_list: Option<Vec<String>>,
}


/// Serde struct used to serialize the body data
/// for a deregister post request
#[derive(Serialize)]
pub struct DeregisterData {
    #[serde(rename(serialize = "correlation-id"))]
    pub(crate) correlation_id: String,

    #[serde(rename(serialize = "secret-key"))]
    pub(crate) secret_key: String,
}


/// Serde struct used to serialize the body data
/// for a register post request
#[derive(Serialize)]
pub struct RegisterData {
    #[serde(rename(serialize = "public-key"))]
    pub(crate) public_key: String,

    #[serde(rename(serialize = "secret-key"))]
    pub(crate) secret_key: String,

    #[serde(rename(serialize = "correlation-id"))]
    pub(crate) correlation_id: String,
}


/// Unit struct used only as an unused generic placeholder
#[derive(Serialize)]
pub struct EmptyData;

pub enum HttpRequest<P: Serialize + Send> {
    Get {
        url: String,
        query_params: SmallVec<[(String, String); 2]>,
        phantom_data: PhantomData<P>,
    },
    Post {
        url: String,
        post_data: P,
    },
}

impl HttpRequest<EmptyData> {
    pub fn new_get_request(
        url: String,
        query_params: SmallVec<[(String, String); 2]>,
    ) -> HttpRequest<EmptyData> {
        let phantom_data: PhantomData<EmptyData> = PhantomData;

        Self::Get {
            url,
            query_params,
            phantom_data,
        }
    }
}

impl<P: Serialize + Send> HttpRequest<P> {
    fn create_request_builder(&self, reqwest_client: &reqwest::Client) -> RequestBuilder {
        match self {
            HttpRequest::Get {
                url, query_params, ..
            } => reqwest_client.get(url).query(query_params),
            HttpRequest::Post { url, post_data } => reqwest_client.post(url).json(&post_data),
        }
    }
}

pub async fn make_http_request<P: Serialize + Send>(
    reqwest_client: &reqwest::Client,
    auth_token: Option<&Secret<String>>,
    request_info: HttpRequest<P>,
) -> Result<Response, reqwest::Error> {
    let mut http_request = request_info.create_request_builder(reqwest_client);

    http_request = match auth_token {
        Some(token) => http_request.header("Authorization", token.expose_secret()),
        None => http_request,
    };

    cfg_if::cfg_if! {
        if #[cfg(feature = "async-compat")] {
            let http_request_future = Compat::new(async {
                http_request.send().await
            });
        } else {
            let http_request_future = http_request.send();
        }
    }

    http_request_future.await
}
