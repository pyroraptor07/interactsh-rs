//! Defines the client struct used for communicating with the Interactsh servers.

use rand::seq::SliceRandom;

use crate::crypto::rsa::RSAPrivKey;

/// The default list of servers provided by the Interactsh team
const DEFAULT_INTERACTSH_SERVERS: &[&str] = &[
    "oast.pro",
    "oast.live",
    "oast.site",
    "oast.online",
    "oast.fun",
    "oast.me",
];

/// Builder for the [Client] struct
pub struct ClientBuilder {
    rsa_key: Option<RSAPrivKey>,
    server: Option<String>,
    auth_token: Option<String>,
}

impl ClientBuilder {
    /// Create a new builder with no options defined
    pub fn new() -> Self {
        Self {
            rsa_key: None,
            server: None,
            auth_token: None,
        }
    }

    /// Create a new builder with the default options
    /// 
    /// This will create a builder with a 2048 bit RSA key, no auth token, and server randomly picked from the
    /// [list of default servers](https://github.com/projectdiscovery/interactsh#using-self-hosted-server) 
    /// provided and maintained by the Interactsh team.
    pub fn default() -> Result<Self, String> {
        let rsa_key = RSAPrivKey::generate(2048)?;
        let server = *DEFAULT_INTERACTSH_SERVERS.choose(&mut rand::thread_rng()).expect("Unable to pick a server from the default list!");

        let new_builder = Self {
            rsa_key: Some(rsa_key),
            server: Some(server.to_string()),
            auth_token: None,
        };

        Ok(new_builder)
    }

    pub fn with_rsa_key_size(self, num_bits: usize) -> Result<Self, String> {
        let rsa_key = RSAPrivKey::generate(num_bits)?;

        let new_builder = Self {
            rsa_key: Some(rsa_key),
            ..self
        };

        Ok(new_builder)
    }

    pub fn with_server(self, server: String) -> Self {
        Self {
            server: Some(server),
            ..self
        }
    }

    pub fn with_auth_token(self, auth_token: String) -> Self {
        Self {
            auth_token: Some(auth_token),
            ..self
        }
    }

    pub fn build(self) -> Result<Client, String> {



        todo!()
    }
}

#[derive(Debug)]
pub struct RegisteredUrl {
    pub sub_domain: String,
    pub server: String,
    pub correlation_id: String,
}

impl RegisteredUrl {
    pub fn full_url(&self) -> String {
        format!("{}.{}", self.sub_domain, self.server)
    }
}

#[derive(Debug)]
pub struct LogEntry {
    pub log: String,
}

pub struct Client {
    rsa_key: RSAPrivKey,
    server: String,
    registered_url: Option<RegisteredUrl>,
    auth_token: Option<String>,
    secret_key: String,
    encoded_pub_key: Vec<u8>,
    reqwest_client: reqwest::Client,
}

impl Client {
    pub async fn register(&mut self) -> Result<&RegisteredUrl, String> {
        todo!()
    }

    pub async fn deregister(&mut self) -> Result<(), String> {
        todo!()
    }

    pub async fn poll(&self) -> Result<Vec<LogEntry>, String> {
        todo!()
    }
}
