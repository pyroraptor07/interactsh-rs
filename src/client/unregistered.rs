use secrecy::{Secret, ExposeSecret};

use crate::crypto::rsa::RSAPrivKey;
use crate::errors::ClientRegistrationError;

use super::{
    client_helpers::{
        self,
        RegisterData,
    },
    builder::AuthToken, 
    registered::RegisteredClient,
};


/// The client type returned by the [ClientBuilder](crate::client::builder::ClientBuilder) 
/// build function.
/// 
/// The register function must be called on this client in order to turn it
/// into a [RegisteredClient](crate::client::registered::RegisteredClient), which can
/// be used to poll an Interactsh server.
#[derive(Debug, Clone)]
pub struct UnregisteredClient {
    pub(crate) rsa_key: RSAPrivKey,
    pub(crate) server: String,
    pub(crate) sub_domain: String,
    pub(crate) correlation_id: String,
    pub(crate) auth_token: Option<AuthToken>,
    pub(crate) secret_key: Secret<String>,
    pub(crate) encoded_pub_key: String,
    pub(crate) reqwest_client: reqwest::Client,
    pub(crate) parse_logs: bool,
}

impl UnregisteredClient {
    /// Registers this client with the Interactsh server it was configured for.
    /// 
    /// On a successful result, this returns a [RegisteredClient](crate::client::registered::RegisteredClient)
    /// that can be used to poll the server. If the registration fails, this returns
    /// a [ClientRegistrationError](crate::errors::client_errors::ClientRegistrationError), which
    /// contains a clone of this client if another try is needed.
    pub async fn register(self) -> Result<RegisteredClient, ClientRegistrationError<UnregisteredClient>> {
        let post_data = RegisterData {
            public_key: self.encoded_pub_key.clone(),
            secret_key: self.secret_key.expose_secret().clone(),
            correlation_id: self.correlation_id.clone(),
        };
        client_helpers::register(
            &self, 
            &post_data,
            format!("https://{}/register", &self.server),
            &self.reqwest_client,
            self.auth_token.as_ref(),
        ).await?;

        let new_reg_client = RegisteredClient {
            rsa_key: self.rsa_key,
            server: self.server,
            sub_domain: self.sub_domain,
            correlation_id: self.correlation_id,
            auth_token: self.auth_token,
            secret_key: self.secret_key,
            reqwest_client: self.reqwest_client,
            parse_logs: self.parse_logs,
        };

        Ok(new_reg_client)
    }
}

impl client_helpers::Client for UnregisteredClient {}
