use std::time::Duration;

use rand::seq::SliceRandom;
use secrecy::Secret;
use svix_ksuid::*;
use uuid::Uuid;

// use super::proxy::ClientProxy;
use super::unregistered::UnregisteredClient;
use crate::crypto::rsa::RSAPrivKey;
use crate::errors::ClientBuildError;

/// The default list of servers provided by the Interactsh team
const DEFAULT_INTERACTSH_SERVERS: &[&str] = &[
    "oast.pro",
    "oast.live",
    "oast.site",
    "oast.online",
    "oast.fun",
    // "oast.me",
];

/// Builds an [UnregisteredClient](crate::client::unregistered::UnregisteredClient)
pub struct ClientBuilder {
    rsa_key_size: Option<usize>,
    server: Option<String>,
    auth_token: Option<Secret<String>>,
    // proxies: Option<Vec<ClientProxy>>,
    timeout: Option<Duration>,
    ssl_verify: bool,
    parse_logs: bool,
}

impl ClientBuilder {
    /// Create a new builder with no options defined.
    pub fn new() -> Self {
        Self {
            rsa_key_size: None,
            server: None,
            auth_token: None,
            // proxies: None,
            timeout: None,
            ssl_verify: false,
            parse_logs: true,
        }
    }

    /// Create a new builder with the default options.
    ///
    /// This will create a builder with a 2048 bit RSA key and server randomly picked from the
    /// [list of default servers](https://github.com/projectdiscovery/interactsh#using-self-hosted-server)
    /// provided and maintained by the Interactsh team. This will also set the timeout
    /// to 15 seconds, SSL verification to false, and parse_logs to true.
    pub fn default() -> Self {
        let server = *DEFAULT_INTERACTSH_SERVERS
            .choose(&mut rand::thread_rng())
            .unwrap_or(&"oast.pro"); // if random choice somehow returns None, just use oast.pro

        Self {
            rsa_key_size: Some(2048),
            server: Some(server.to_string()),
            auth_token: None,
            // proxies: None,
            timeout: Some(Duration::from_secs(15)),
            ssl_verify: false,
            parse_logs: true,
        }
    }

    pub fn with_rsa_key_size(self, num_bits: usize) -> Self {
        Self {
            rsa_key_size: Some(num_bits),
            ..self
        }
    }

    /// Set the Interactsh server that the client will connect to.
    pub fn with_server(self, server: String) -> Self {
        Self {
            server: Some(server),
            ..self
        }
    }

    /// Set an optional auth token that the client will use to authenticate
    /// with the Interactsh server.
    ///
    /// If this is not set, then no auth header will be sent to the
    /// server.
    pub fn with_auth_token(self, auth_token: String) -> Self {
        let token = Secret::new(auth_token);
        Self {
            auth_token: Some(token),
            ..self
        }
    }

    // /// Set an optional proxy URL that the client can use.
    // ///
    // /// This can be set more than once; each new proxy URL will be added
    // /// to a list of proxies that the client will try.
    // pub fn with_proxy(self, proxy: ClientProxy) -> Self {
    //     let proxies = match self.proxies {
    //         Some(mut proxies) => {
    //             proxies.push(proxy);
    //             Some(proxies)
    //         }
    //         None => Some(vec![proxy]),
    //     };

    //     Self { proxies, ..self }
    // }

    /// Set the timeout value for server requests.
    pub fn with_timeout(self, timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            ..self
        }
    }

    /// Sets whether or not the client should verify the
    /// server's SSL certificate.
    pub fn verify_ssl(self, ssl_verify: bool) -> Self {
        Self { ssl_verify, ..self }
    }

    /// Sets whether or not the client should parse the logs
    /// or just return the raw logs.
    pub fn parse_logs(self, parse_logs: bool) -> Self {
        Self { parse_logs, ..self }
    }

    /// Builds an [UnregisteredClient](crate::client::unregistered::UnregisteredClient).
    ///
    /// The server must be set and the RSA key generated in order for
    /// this to succeed. If the build succeeds, the
    /// register function must be called on the returned
    ///  [UnregisteredClient](crate::client::unregistered::UnregisteredClient)
    /// to turn it into a [RegisteredClient](crate::client::registered::RegisteredClient).
    pub fn build(self) -> Result<UnregisteredClient, ClientBuildError> {
        // Ensure rsa_key and server are set
        let rsa_key_size = self
            .rsa_key_size
            .ok_or(ClientBuildError::MissingRsaKeySize)?;
        let server = self.server.ok_or(ClientBuildError::MissingServer)?;

        // Get the other values needed
        let rsa_key = RSAPrivKey::generate(rsa_key_size)?;
        let pubkey = rsa_key.get_pub_key()?;
        let secret = Uuid::new_v4().to_string();
        let encoded_pub_key = pubkey.b64_encode()?;
        let ksuid_a = Ksuid::new(None, None).to_string().to_ascii_lowercase();
        let ksuid_b = Ksuid::new(None, None).to_string().to_ascii_lowercase();
        let mut sub_domain = format!("{}{}", ksuid_a, ksuid_b);
        sub_domain.truncate(33);

        let mut correlation_id = sub_domain.clone();
        correlation_id.truncate(20);

        // Build the reqwest client
        let mut reqwest_client_builder = reqwest::Client::builder();

        // reqwest_client_builder = match self.proxies {
        //     None => reqwest_client_builder,
        //     Some(proxies) => {
        //         let mut builder = reqwest_client_builder;

        //         for proxy in proxies.into_iter() {
        //             builder = builder.proxy(proxy.into_reqwest_proxy()?);
        //         }

        //         builder
        //     }
        // };

        let timeout = self.timeout.unwrap_or(Duration::from_secs(15));
        reqwest_client_builder = reqwest_client_builder.timeout(timeout);

        cfg_if::cfg_if! {
            if #[cfg(all(feature = "reqwest-rustls-tls", feature = "reqwest-native-tls"))] {
                reqwest_client_builder = reqwest_client_builder.use_rustls_tls();
            }
        }

        reqwest_client_builder =
            reqwest_client_builder.danger_accept_invalid_certs(!self.ssl_verify);

        let reqwest_client = reqwest_client_builder.build()?;

        // Create the UnregisteredClient object
        let unreg_client = UnregisteredClient {
            rsa_key,
            server,
            sub_domain,
            correlation_id,
            auth_token: self.auth_token,
            secret_key: Secret::new(secret),
            encoded_pub_key,
            reqwest_client,
            parse_logs: self.parse_logs,
        };

        Ok(unreg_client)
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::default()
    }
}
