#[cfg(feature = "async-compat")]
use async_compat::Compat;
use reqwest::StatusCode;
use secrecy::ExposeSecret;

use crate::errors::{
    ClientRegistrationError,
    ClientRegistrationInnerError,
};

use super::builder::AuthToken;




#[derive(serde::Deserialize)]
pub(crate) struct PollResponse {
    pub(crate) aes_key: String,

    #[serde(rename(deserialize = "data"))]
    pub(crate) data_list: Vec<String>,
}


pub trait PostData {}


#[derive(serde::Serialize)]
pub struct DeregisterData {
    #[serde(rename(serialize = "correlation-id"))]
    pub(crate) correlation_id: String,

    #[serde(rename(serialize = "secret-key"))]
    pub(crate) secret_key: String,
}

impl PostData for DeregisterData {}


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


pub trait Client {
    fn get_reqwest_client(&self) -> &reqwest::Client;
    fn get_auth_token(&self) -> Option<&AuthToken>;
}


pub async fn register<T: Client + Clone, D: PostData + serde::Serialize>(
    client: &T,
    post_data: &D,
    register_url: String,
) -> Result<(), ClientRegistrationError<T>> {
    let mut post_request = client.get_reqwest_client().post(register_url);

    post_request = match client.get_auth_token() {
        Some(AuthToken::SimpleAuth(token)) => post_request.header("Authorization", token.expose_secret()),
        Some(AuthToken::BearerAuth(token)) => post_request.bearer_auth(token.expose_secret()),
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

    let register_response = post_request_future.await
        .map_err(|e| {
            let inner_error = ClientRegistrationInnerError::from(e);
            ClientRegistrationError::new(
                client.clone(),
                inner_error,
            )
        })?;

    match register_response.status() {
        StatusCode::OK => Ok(()),
        StatusCode::UNAUTHORIZED => {
            let inner_error = ClientRegistrationInnerError::Unauthorized;
            let error = ClientRegistrationError::new(
                client.clone(),
                inner_error,
            );

            Err(error)
        }
        status => {
            let server_msg = register_response.text().await.unwrap_or("Unknown error".to_string());
            let status_code = status.as_u16();

            let inner_error = ClientRegistrationInnerError::RegistrationFailure { 
                server_msg,
                status_code,
            };

            let error = ClientRegistrationError::new(
                client.clone(),
                inner_error,
            );

            Err(error)
        }
    }
}
