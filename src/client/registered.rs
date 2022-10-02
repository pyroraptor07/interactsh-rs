#[cfg(feature = "async-compat")]
use async_compat::Compat;
use secrecy::{Secret, ExposeSecret};

use crate::crypto::aes;
use crate::crypto::rsa::RSAPrivKey;
use crate::errors::{ClientError, ClientRegistrationError};

use super::{
    client_helpers::{
        self,
        DeregisterData,
        PollResponse,
    },
    builder::AuthToken,
    interaction_log::LogEntry,
};


/// The client type returned when an [UnregisteredClient](crate::client::unregistered::UnregisteredClient)
/// successfully registers with its configured Interactsh server.
#[derive(Debug, Clone)]
pub struct RegisteredClient {
    pub(crate) rsa_key: RSAPrivKey,
    pub(crate) server: String,
    pub(crate) sub_domain: String,
    pub(crate) correlation_id: String,
    pub(crate) auth_token: Option<AuthToken>,
    pub(crate) secret_key: Secret<String>,
    pub(crate) reqwest_client: reqwest::Client,
    pub(crate) parse_logs: bool,
}

impl RegisteredClient {
    /// Gets the interaction URL for the current
    /// registered session
    pub fn get_interaction_url(&self) -> String {
        format!("{}.{}", self.sub_domain, self.server)
    }

    /// Deregisters the [RegisteredClient] with the Interactsh server.
    /// 
    /// If the deregistration fails, this returns a 
    /// [ClientRegistrationError](crate::errors::client_errors::ClientRegistrationError),
    /// which contains a clone of this client if another try is needed.
    pub async fn deregister(self) -> Result<(), ClientRegistrationError<RegisteredClient>> {
        let post_data = DeregisterData {
            correlation_id: self.correlation_id.clone(),
            secret_key: self.secret_key.expose_secret().clone(),
        };
        client_helpers::register(
            &self,
            &post_data,
            format!("https://{}/deregister", &self.server),
            &self.reqwest_client,
            self.auth_token.as_ref(),
        ).await?;

        Ok(())
    }

    /// Polls the Interactsh server for any new logs.
    /// 
    /// This returns a vec of [LogEntry](crate::client::interaction_log::LogEntry).
    /// If there are no new logs, an empty vec is returned.
    pub async fn poll(&self) -> Result<Vec<LogEntry>, ClientError> {
        let poll_url = format!("https://{}/poll", self.server);
        let req_query_params = &[
            ("id", &self.correlation_id),
            ("secret", &self.secret_key.expose_secret()),
        ];

        let mut get_request = self.reqwest_client
            .get(poll_url)
            .query(req_query_params);
        
        get_request = match &self.auth_token {
            Some(AuthToken::SimpleAuth(token)) => get_request.header("Authorization", token.expose_secret()),
            Some(AuthToken::BearerAuth(token)) => get_request.bearer_auth(token.expose_secret()),
            None => get_request,
        };

        cfg_if::cfg_if! {
            if #[cfg(feature = "async-compat")] {
                let get_request_future = Compat::new(async {
                    get_request.send().await
                });
            } else {
                let get_request_future = get_request.send();
            }
        }

        let get_response = get_request_future.await?;
        let status = &get_response.status();

        if !status.is_success() {
            let server_msg = get_response.text().await.unwrap_or("Unknown error".to_string());
            let status_code = status.as_u16();
            let error = ClientError::PollError {
                server_msg,
                status_code,
            };

            return Err(error);
        }

        let response_body = get_response.json::<PollResponse>().await?;
        let aes_key_decoded = base64::decode(&response_body.aes_key)?;
        
        let mut results = Vec::new();
        for data in response_body.data_list.iter() {
            let data_decoded = base64::decode(data)?;
            let decrypted_data = self.decrypt_data(&aes_key_decoded, &data_decoded)?;
            
            let log_entry = if self.parse_logs {
                LogEntry::try_parse_log(decrypted_data.as_str())
            } else {
                LogEntry::return_raw_log(decrypted_data.as_str())
            };
            
            results.push(log_entry);
        }

        Ok(results)
    }

    fn decrypt_data(&self, aes_key: &Vec<u8>, encrypted_data: &Vec<u8>) -> Result<String, ClientError> {
        let aes_plain_key = self.rsa_key.decrypt_data(aes_key)?;
        let decrypted_data = aes::decrypt_data(&aes_plain_key, encrypted_data)?;
        let decrypted_string = String::from_utf8_lossy(&decrypted_data);

        Ok(decrypted_string.into())
    }
}

impl client_helpers::Client for RegisteredClient {}
