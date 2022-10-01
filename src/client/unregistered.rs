#[cfg(feature = "async-compat")]
use async_compat::Compat;
use reqwest::StatusCode;

use crate::crypto::rsa::RSAPrivKey;
use crate::errors::{ClientRegistrationError, ClientRegistrationInnerError};

use super::{builder::AuthToken, registered::Client};


#[derive(serde::Serialize, Debug)]
struct RegisterData {
    #[serde(rename(serialize = "public-key"))]
    public_key: String,

    #[serde(rename(serialize = "secret-key"))]
    secret_key: String,

    #[serde(rename(serialize = "correlation-id"))]
    correlation_id: String,
}


#[derive(Debug, Clone)]
pub struct UnregisteredClient {
    pub(crate) rsa_key: RSAPrivKey,
    pub(crate) server: String,
    pub(crate) sub_domain: String,
    pub(crate) correlation_id: String,
    pub(crate) auth_token: Option<AuthToken>,
    pub(crate) secret_key: String,
    pub(crate) encoded_pub_key: String,
    pub(crate) reqwest_client: reqwest::Client,
    pub(crate) parse_logs: bool,
}

impl UnregisteredClient {
    pub async fn register(self) -> Result<Client, ClientRegistrationError> {
        let data = RegisterData {
            public_key: self.encoded_pub_key.clone(),
            secret_key: self.secret_key.clone(),
            correlation_id: self.correlation_id.clone(),
        };

        let register_url = format!("https://{}/register", self.server);
        let mut post_request = self.reqwest_client.post(register_url);

        post_request = match &self.auth_token {
            Some(AuthToken::SimpleAuth(token)) => post_request.header("Authorization", token),
            Some(AuthToken::BearerAuth(token)) => post_request.bearer_auth(token),
            None => post_request,
        };

        post_request = post_request.json(&data);

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
                
                ClientRegistrationError {
                    error: inner_error,
                    unregistered_client: self.clone(),
                }
            })?;

        match register_response.status() {
            StatusCode::OK => {
                let new_reg_client = Client {
                    rsa_key: self.rsa_key,
                    server: self.server,
                    sub_domain: self.sub_domain,
                    correlation_id: self.correlation_id,
                    auth_token: self.auth_token,
                    secret_key: self.secret_key,
                    encoded_pub_key: self.encoded_pub_key,
                    reqwest_client: self.reqwest_client,
                    parse_logs: self.parse_logs,
                };

                Ok(new_reg_client)
            }
            StatusCode::UNAUTHORIZED => {
                let inner_error = ClientRegistrationInnerError::Unauthorized;
                let error = ClientRegistrationError {
                    error: inner_error,
                    unregistered_client: self.clone(),
                };
                Err(error)
            }
            status => {
                let server_msg = register_response.text().await.unwrap_or("Unknown error".to_string());
                let status_code = status.as_u16();

                let inner_error = ClientRegistrationInnerError::RegistrationFailure { 
                    server_msg,
                    status_code,
                };

                let error = ClientRegistrationError {
                    error: inner_error,
                    unregistered_client: self.clone(),
                };

                Err(error)
            }
        }
    }
}
