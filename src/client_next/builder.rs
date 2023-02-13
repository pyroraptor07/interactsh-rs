use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use async_lock::RwLock;
use cfg_if::cfg_if;
use rand::seq::SliceRandom;
use reqwest::Proxy;
use secrecy::Secret;
use snafu::{OptionExt, ResultExt};
use uuid::Uuid;

use super::{CorrelationConfig, InteractshClient};
use crate::client_shared::errors::{client_build_error, ClientBuildError};
use crate::client_shared::server_comm::{ClientStatus, ServerComm};
use crate::crypto::rsa::RSAPrivKey;

/// The default list of servers provided by the Interactsh team
const DEFAULT_INTERACTSH_SERVERS: &[&str] = &[
    "oast.pro",
    "oast.live",
    "oast.site",
    "oast.online",
    "oast.fun",
    // "oast.me",
];

#[allow(unused)]
enum RsaKeyGen {
    BuilderGen(usize),
    UserProvided(Box<RSAPrivKey>),
}

enum TlsOption {
    #[cfg(feature = "rustls-tls")]
    Rustls,
    #[cfg(feature = "native-tls")]
    Native,
}

impl Default for TlsOption {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(feature = "rustls-tls")] {
                Self::Rustls
            } else if #[cfg(feature = "native-tls")] {
                Self::Native
            } else {
                panic!("A TLS feature option must be selected!")
            }
        }
    }
}

#[allow(unused)]
pub struct ClientBuilder {
    rsa_key_gen: Option<RsaKeyGen>,
    server: Option<String>,
    auth_token: Option<Secret<String>>,
    correlation_config: Option<CorrelationConfig>,
    tls_option: TlsOption,
    proxies: Option<Vec<Proxy>>,
    timeout: Option<Duration>,
    ssl_verify: bool,
    parse_logs: bool,
    dns_override: Option<IpAddr>,
}

impl ClientBuilder {
    /// Create a new builder with no options defined.
    pub fn new() -> Self {
        Self {
            rsa_key_gen: None,
            server: None,
            auth_token: None,
            correlation_config: None,
            tls_option: TlsOption::default(),
            proxies: None,
            timeout: None,
            ssl_verify: false,
            parse_logs: true,
            dns_override: None,
        }
    }

    /// Sets the RSA key size that the builder will generate for the client.
    pub fn with_rsa_key_size(self, num_bits: usize) -> Self {
        Self {
            rsa_key_gen: Some(RsaKeyGen::BuilderGen(num_bits)),
            ..self
        }
    }

    /// Provides an existing RSA private key for the client to use.
    // pub fn with_existing_rsa_key(self, rsa_key: RSAPrivKey) -> Self {
    //     Self {
    //         rsa_key_gen: Some(RsaKeyGen::UserProvided(Box::new(rsa_key))),
    //         ..self
    //     }
    // }

    /// Sets the Interactsh server that the client will connect to.
    pub fn with_server(self, server: String) -> Self {
        Self {
            server: Some(server),
            ..self
        }
    }

    /// Sets an optional auth token that the client will use to authenticate
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

    /// Takes in an optional [CorrelationConfig](super::CorrelationConfig) that
    /// can be used to set the subdomain length and correlation ID length. This
    /// must match what the Interactsh server is configured for. If omitted,
    /// the client will use the defaults of 33 for the subdomain and
    /// 20 for the correlation ID, which is needed for the public Interactsh servers.
    pub fn with_correlation_config(self, config: CorrelationConfig) -> Self {
        Self {
            correlation_config: Some(config),
            ..self
        }
    }

    /// Force the client to use reqwest's native TLS backend.
    ///
    /// Does nothing if only the `native-tls` TLS feature is enabled.
    #[cfg(feature = "native-tls")]
    pub fn use_native_tls(self) -> Self {
        Self {
            tls_option: TlsOption::Native,
            ..self
        }
    }

    /// Force the client to use reqwest's rustls TLS backend
    ///
    /// Does nothing if only the `rustls-tls` TLS feature is enabled.
    #[cfg(feature = "rustls-tls")]
    pub fn use_rustls_tls(self) -> Self {
        Self {
            tls_option: TlsOption::Rustls,
            ..self
        }
    }

    /// Sets an optional proxy that the client can use.
    ///
    /// This can be set more than once; each new proxy will be added
    /// to a list of proxies that the client will try. Proxies will be
    /// tried in the order added.
    pub fn with_proxy(self, proxy: Proxy) -> Self {
        let proxies = match self.proxies {
            Some(mut proxies) => {
                proxies.push(proxy);
                Some(proxies)
            }
            None => Some(vec![proxy]),
        };

        Self { proxies, ..self }
    }

    /// Sets the timeout value for server requests.
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

    /// Sets an option on the client to override normal DNS
    /// resolution for the server and instead use the provided
    /// IP address.
    pub fn set_dns_override(self, server_ip_address: IpAddr) -> Self {
        Self {
            dns_override: Some(server_ip_address),
            ..self
        }
    }

    pub fn build(self) -> Result<InteractshClient, ClientBuildError> {
        // Ensure server name and rsa key options were set
        let server = self
            .server
            .context(client_build_error::MissingServerSnafu)?;
        let rsa_key_gen = self
            .rsa_key_gen
            .context(client_build_error::MissingRsaKeyOptionSnafu)?;

        // Generate RSA key pair and secret
        let rsa_key = match rsa_key_gen {
            RsaKeyGen::BuilderGen(rsa_key_size) => {
                RSAPrivKey::generate(rsa_key_size).context(client_build_error::RsaGenSnafu)?
            }
            RsaKeyGen::UserProvided(rsa_key) => *rsa_key,
        };
        let pub_key = rsa_key
            .get_pub_key()
            .context(client_build_error::PubKeyExtractSnafu)?
            .b64_encode()
            .context(client_build_error::PubKeyEncodeSnafu)?;
        let secret = Uuid::new_v4().to_string();

        // Build the reqwest client
        let mut reqwest_client_builder = reqwest::Client::builder();

        reqwest_client_builder = match self.proxies {
            None => reqwest_client_builder,
            Some(proxies) => {
                let mut builder = reqwest_client_builder;

                for proxy in proxies.into_iter() {
                    builder = builder.proxy(proxy);
                }

                builder
            }
        };

        let timeout = self.timeout.unwrap_or(Duration::from_secs(15));
        reqwest_client_builder = reqwest_client_builder.timeout(timeout);

        cfg_if! {
            if #[cfg(all(feature = "reqwest-rustls-tls", feature = "reqwest-native-tls"))] {
                reqwest_client_builder = match self.tls_option {
                    TlsOption::Native => reqwest_client_builder.use_native_tls(),
                    TlsOption::Rustls => reqwest_client_builder.use_rustls_tls(),
                };
            }
        }


        reqwest_client_builder =
            reqwest_client_builder.danger_accept_invalid_certs(!self.ssl_verify);

        reqwest_client_builder = match self.dns_override {
            Some(server_ip_address) => {
                let socket_addr = SocketAddr::new(server_ip_address, 443);
                reqwest_client_builder.resolve(server.as_str(), socket_addr)
            }
            None => reqwest_client_builder,
        };

        let reqwest_client = reqwest_client_builder
            .build()
            .context(client_build_error::ReqwestBuildFailedSnafu)?;

        // Build the internal ServerComm object
        let server_comm = ServerComm {
            server_name: server,
            auth_token: self.auth_token,
            secret_key: Secret::new(secret),
            encoded_pub_key: pub_key,
            reqwest_client: Arc::new(reqwest_client),
            correlation_config: self.correlation_config,
            status: ClientStatus::Unregistered,
        };

        // Return the new client
        let client = InteractshClient {
            rsa_key: Arc::new(rsa_key),
            server_comm: Arc::new(RwLock::new(server_comm)),
            parse_logs: self.parse_logs,
        };

        Ok(client)
    }
}

impl Default for ClientBuilder {
    /// Create a new builder with the default options.
    ///
    /// This will create a builder with a 2048 bit RSA key and server randomly picked from the
    /// [list of default servers](https://github.com/projectdiscovery/interactsh#using-self-hosted-server)
    /// provided and maintained by the Interactsh team. This will also set the timeout
    /// to 15 seconds, SSL verification to false, and parse_logs to true.
    fn default() -> Self {
        let server = *DEFAULT_INTERACTSH_SERVERS
            .choose(&mut rand::thread_rng())
            .unwrap_or(&"oast.pro"); // if random choice somehow returns None, just use oast.pro

        Self {
            rsa_key_gen: Some(RsaKeyGen::BuilderGen(2048)),
            server: Some(server.to_string()),
            auth_token: None,
            correlation_config: None,
            tls_option: TlsOption::default(),
            proxies: None,
            timeout: Some(Duration::from_secs(15)),
            ssl_verify: false,
            parse_logs: true,
            dns_override: None,
        }
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rand::{Rng, RngCore};

    use super::*;

    #[test]
    fn default_build_succeeds() {
        let _builder = ClientBuilder::default()
            .build()
            .expect("Default build failed");
    }

    #[test]
    fn empty_builder_fails() {
        let _builder = ClientBuilder::new()
            .build()
            .expect_err("Empty builder did not fail as expected");
    }

    #[test]
    fn build_with_server_and_rsa_only_succeeds() {
        let _builder = ClientBuilder::new()
            .with_server("oast.pro".into())
            .with_rsa_key_size(2048)
            .build()
            .expect("Build with only server and rsa failed");
    }

    #[test]
    // Note: does not test dns override; that is tested in integration testing
    fn build_with_all_options_succeeds() {
        let mut rng = rand::thread_rng();

        // Generate a random token string
        let mut rand_bytes: [u8; 32] = [0; 32];
        rng.fill_bytes(&mut rand_bytes);
        let token = hex::encode(rand_bytes);

        // Get a random duration in seconds
        let duration_secs = rng.gen_range(5..=30);

        // Generate boolean values
        let verify_ssl = rng.gen_bool(1.0 / 2.0);
        let parse_logs = rng.gen_bool(1.0 / 2.0);

        let _builder = ClientBuilder::new()
            .with_server("oast.pro".into())
            .with_rsa_key_size(2048)
            .with_auth_token(token)
            .with_timeout(Duration::from_secs(duration_secs))
            .verify_ssl(verify_ssl)
            .parse_logs(parse_logs)
            .build()
            .expect("Build with all options failed");
    }

    #[test]
    fn build_with_only_server_fails() {
        let _builder = ClientBuilder::new()
            .with_server("oast.pro".into())
            .build()
            .expect_err("Server-only build did not fail as expected");
    }

    #[test]
    fn build_with_only_rsa_fails() {
        let _builder = ClientBuilder::new()
            .with_rsa_key_size(2048)
            .build()
            .expect_err("RSA-only build did not fail as expected");
    }
}
