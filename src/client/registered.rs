use async_compat::Compat;

use crate::crypto::aes;
use crate::crypto::rsa::RSAPrivKey;
use crate::errors::ClientError;

use super::{builder::AuthToken, interaction_log::LogEntry};


#[derive(serde::Deserialize)]
struct PollResponse {
    aes_key: String,

    #[serde(rename(deserialize = "data"))]
    data_list: Vec<String>,
}


/// The primary struct used to communicate with an
/// Interactsh server.
#[allow(unused)]
pub struct Client {
    pub(crate) rsa_key: RSAPrivKey,
    pub(crate) server: String,
    pub(crate) sub_domain: String,
    pub(crate) correlation_id: String,
    pub(crate) auth_token: Option<AuthToken>,
    pub(crate) secret_key: String,
    pub(crate) encoded_pub_key: String,
    pub(crate) reqwest_client: reqwest::Client,
}

impl Client {
    /// Gets the URL that the [Client] registered to the 
    /// Interactsh server with.
    pub fn get_registered_url(&self) -> String {
        format!("{}.{}", self.sub_domain, self.server)
    }

    /// Deregisters the [Client] with the Interactsh server.
    /// 
    /// If this is successful, then a new [Client] must be created
    /// to re-register.
    pub async fn deregister(&mut self) -> Result<(), ClientError> {
        todo!()
    }

    /// Polls the Interactsh server for any new logs.
    pub async fn poll(&self) -> Result<Vec<LogEntry>, ClientError> {
        let poll_url = format!("https://{}/poll", self.server);
        let req_query_params = &[
            ("id", &self.correlation_id),
            ("secret", &self.secret_key),
        ];

        let mut get_request = self.reqwest_client
            .get(poll_url)
            .query(req_query_params);
        
        get_request = match &self.auth_token {
            Some(AuthToken::SimpleAuth(token)) => get_request.header("Authorization", token),
            Some(AuthToken::BearerAuth(token)) => get_request.bearer_auth(token),
            None => get_request,
        };

        let get_response = Compat::new(async {
            get_request.send().await
        }).await?;
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
        let aes_key_decoded = base64::decode(&response_body.aes_key).unwrap();
        
        let mut results = Vec::new();
        for data in response_body.data_list.iter() {
            let data_decoded = base64::decode(data).unwrap();
            let decrypted_data = self.decrypt_data(&aes_key_decoded, &data_decoded)?;
            let log_entry = serde_json::from_str::<LogEntry>(decrypted_data.as_str()).unwrap();
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
