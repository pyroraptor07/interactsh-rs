#[cfg(feature = "async-compat")]
use async_compat::Compat;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, Secret};
use snafu::ResultExt;

use super::errors::{registration_error, RegistrationError};

// Marker traits

/// Marker trait for the types used to
/// serialize post request body data
pub trait PostData {}

/// Marker trait for the
/// [UnregisteredClient](crate::client::unregistered::UnregisteredClient) and
/// [RegisteredClient](crate::client::registered::RegisteredClient) types
pub trait Client {
    type PostData;
}

// Serde objects

/// Serde struct used to deserialize the
/// json body of a poll response
#[derive(serde::Deserialize)]
pub(crate) struct PollResponse {
    pub(crate) aes_key: String,

    #[serde(rename(deserialize = "data"))]
    pub(crate) data_list: Option<Vec<String>>,
}

/// Serde struct used to serialize the body data
/// for a deregister post request
#[derive(serde::Serialize)]
pub struct DeregisterData {
    #[serde(rename(serialize = "correlation-id"))]
    pub(crate) correlation_id: String,

    #[serde(rename(serialize = "secret-key"))]
    pub(crate) secret_key: String,
}

impl PostData for DeregisterData {}

/// Serde struct used to serialize the body data
/// for a register post request
#[derive(serde::Serialize)]
pub struct RegisterData {
    #[serde(rename(serialize = "public-key"))]
    pub(crate) public_key: String,

    #[serde(rename(serialize = "secret-key"))]
    pub(crate) secret_key: String,

    #[serde(rename(serialize = "correlation-id"))]
    pub(crate) correlation_id: String,
}

impl PostData for RegisterData {}

/// Sends a post request to register or deregister a [Client]
pub(crate) async fn register<T>(
    post_data: &T::PostData,
    register_url: String,
    reqwest_client: &reqwest::Client,
    auth_token: Option<&Secret<String>>,
) -> Result<(), RegistrationError>
where
    T: Client + Clone,
    T::PostData: serde::Serialize,
{
    let mut post_request = reqwest_client.post(register_url);

    post_request = match auth_token {
        Some(token) => post_request.header("Authorization", token.expose_secret()),
        None => post_request,
    };

    post_request = post_request.json(post_data);

    cfg_if::cfg_if! {
        if #[cfg(feature = "async-compat")] {
            let post_request_future = Compat::new(async {
                post_request.send().await
            });
        } else {
            let post_request_future = post_request.send();
        }
    }

    let register_response = post_request_future
        .await
        .context(registration_error::RequestSendFailure)?;

    match register_response.status() {
        StatusCode::OK => Ok(()),
        StatusCode::UNAUTHORIZED => registration_error::Unauthorized.fail(),
        status => {
            let server_msg = register_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let status_code = status.as_u16();

            let error = registration_error::RegistrationFailure {
                server_msg,
                status_code,
            };

            error.fail()
        }
    }
}
