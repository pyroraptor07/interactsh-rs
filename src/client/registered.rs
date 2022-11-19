use secrecy::{ExposeSecret, Secret};
use smallvec::SmallVec;
use snafu::ResultExt;

use super::errors::{
    client_poll_error,
    client_registration_error,
    ClientPollError,
    ClientRegistrationError,
};
use super::http_utils::{self, Client, DeregisterData, HttpRequest, PollResponse};
use crate::crypto::aes;
use crate::crypto::rsa::RSAPrivKey;
use crate::interaction_log::LogEntry;

/// The client type returned when an [UnregisteredClient](crate::client::UnregisteredClient)
/// successfully registers with its configured Interactsh server.
#[derive(Debug, Clone)]
pub struct RegisteredClient {
    pub(crate) rsa_key: RSAPrivKey,
    pub(crate) server: String,
    pub(crate) sub_domain: String,
    pub(crate) correlation_id: String,
    pub(crate) auth_token: Option<Secret<String>>,
    pub(crate) secret_key: Secret<String>,
    pub(crate) reqwest_client: reqwest::Client,
    pub(crate) parse_logs: bool,
}

impl RegisteredClient {
    #[deprecated(
        since = "0.2",
        note = "Renaming for accuracy. Use get_interaction_fqdn() instead."
    )]
    /// Gets the interaction FQDN for the current
    /// registered session
    ///
    /// For naming accuracy, this function has been replaced by [get_interaction_fqdn()](RegisteredClient::get_interaction_fqdn()).
    pub fn get_interaction_url(&self) -> String {
        self.get_interaction_fqdn()
    }

    /// Gets the interaction FQDN for the current
    /// registered session
    pub fn get_interaction_fqdn(&self) -> String {
        format!("{}.{}", self.sub_domain, self.server)
    }

    /// Deregisters the [RegisteredClient] with the Interactsh server.
    ///
    /// If the deregistration fails, this returns a
    /// [ClientRegistrationError](super::errors::ClientRegistrationError),
    /// which contains a clone of this client if another try is needed.
    pub async fn deregister(self) -> Result<(), ClientRegistrationError<RegisteredClient>> {
        let post_data = DeregisterData {
            correlation_id: self.correlation_id.clone(),
            secret_key: self.secret_key.expose_secret().clone(),
        };

        self.do_registration_request(post_data)
            .await
            .context(client_registration_error::ClientRegistration { client: self })?;

        Ok(())
    }

    /// Polls the Interactsh server for any new logs.
    pub async fn poll(&self) -> Result<Option<Vec<LogEntry>>, ClientPollError> {
        let poll_url = format!("https://{}/poll", self.server);

        let mut query_params = SmallVec::<[(String, String); 2]>::new();
        query_params.push(("id".into(), self.correlation_id.clone()));
        query_params.push(("secret".into(), self.secret_key.expose_secret().clone()));

        let request_info = HttpRequest::new_get_request(poll_url, query_params);

        let get_response = http_utils::make_http_request(
            &self.reqwest_client,
            self.auth_token.as_ref(),
            request_info,
        )
        .await
        .context(client_poll_error::PollFailure)?;

        let status = &get_response.status();

        if !status.is_success() {
            let server_msg = get_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let status_code = status.as_u16();
            let error = client_poll_error::PollErrorStatus {
                server_msg,
                status_code,
            };

            return error.fail();
        }

        let response_body = get_response
            .json::<PollResponse>()
            .await
            .context(client_poll_error::ResponseJsonParseFailed)?;

        let response_body_data = match response_body.data_list {
            Some(data) => {
                if data.is_empty() {
                    return Ok(None);
                } else {
                    data
                }
            }
            None => return Ok(None),
        };
        let aes_key_decoded = base64::decode(&response_body.aes_key)
            .context(client_poll_error::Base64DecodeFailed)?;

        let mut results = Vec::new();
        for data in response_body_data.iter() {
            let data_decoded =
                base64::decode(data).context(client_poll_error::Base64DecodeFailed)?;
            let decrypted_data = self.decrypt_data(&aes_key_decoded, &data_decoded)?;

            let log_entry = if self.parse_logs {
                LogEntry::try_parse_log(decrypted_data.as_str())
            } else {
                LogEntry::return_raw_log(decrypted_data.as_str())
            };

            results.push(log_entry);
        }

        Ok(Some(results))
    }

    fn decrypt_data(
        &self,
        aes_key: &[u8],
        encrypted_data: &[u8],
    ) -> Result<String, ClientPollError> {
        let aes_plain_key = self
            .rsa_key
            .decrypt_data(aes_key)
            .context(client_poll_error::AesKeyDecryptFailed)?;

        let decrypted_data = aes::decrypt_data(&aes_plain_key, encrypted_data)
            .context(client_poll_error::DataDecryptFailed)?;

        let decrypted_string = String::from_utf8_lossy(&decrypted_data);

        Ok(decrypted_string.into())
    }
}

impl Client for RegisteredClient {
    fn get_registration_url(&self) -> String {
        format!("https://{}/deregister", &self.server)
    }

    fn get_reqwest_client(&self) -> &reqwest::Client {
        &self.reqwest_client
    }

    fn get_auth_token(&self) -> Option<&Secret<String>> {
        self.auth_token.as_ref()
    }
}
