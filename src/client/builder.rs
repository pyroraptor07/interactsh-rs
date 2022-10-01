use std::time::Duration;

use rand::seq::SliceRandom;
use uuid::Uuid;
use svix_ksuid::*;

use crate::crypto::rsa::RSAPrivKey;
use crate::errors::ClientBuildError;

use super::{proxy::ClientProxy, unregistered::UnregisteredClient};


/// The default list of servers provided by the Interactsh team
const DEFAULT_INTERACTSH_SERVERS: &[&str] = &[
    "oast.pro",
    "oast.live",
    "oast.site",
    "oast.online",
    "oast.fun",
    // "oast.me",
];


/// Represents an auth token to use when authenticating with an
/// Interactsh server.
/// 
/// Use SimpleAuth if the server does not follow the 
/// [HTTP bearer authentication format](https://swagger.io/docs/specification/authentication/bearer-authentication/)
/// and instead uses the format `Authorization: <token>`.
/// 
/// Use BearerAuth if the server does follow the HTTP
/// bearer authentication format of `Authorization: Bearer <token>`.
#[derive(Debug, Clone)]
pub enum AuthToken {
    SimpleAuth(String),
    BearerAuth(String),
}

/// Builder for the [Client] struct
pub struct ClientBuilder {
    rsa_key_size: Option<usize>,
    server: Option<String>,
    auth_token: Option<AuthToken>,
    proxies: Option<Vec<ClientProxy>>,
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
            proxies: None,
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
    /// to 15 seconds and SSL verification to false.
    pub fn default() -> Self {
        let server = *DEFAULT_INTERACTSH_SERVERS.choose(&mut rand::thread_rng()).expect("Unable to pick a server from the default list!");

        let new_builder = Self {
            rsa_key_size: Some(2048),
            server: Some(server.to_string()),
            auth_token: None,
            proxies: None,
            timeout: Some(Duration::from_secs(15)),
            ssl_verify: false,
            parse_logs: true,
        };

        new_builder
    }

    /// Generate a new RSA private key for the [Client] to use of the bit size given.
    // pub fn gen_rsa_key(self, num_bits: usize) -> Result<Self, ClientBuildError> {
    //     let rsa_key = RSAPrivKey::generate(num_bits)?;

    //     let new_builder = Self {
    //         rsa_key: Some(rsa_key),
    //         ..self
    //     };

    //     Ok(new_builder)
    // }

    pub fn with_rsa_key_size(self, num_bits: usize) -> Self {
        Self {
            rsa_key_size: Some(num_bits),
            ..self
        }
    }

    /// Set a predefined RSA private key for the [Client] to use.
    // pub fn with_rsa_key(self, rsa_key: RSAPrivKey) -> Self {
    //     Self {
    //         rsa_key: Some(rsa_key),
    //         ..self
    //     }
    // }

    /// Set the Interactsh server that the [Client] will connect to.
    pub fn with_server(self, server: String) -> Self {
        Self {
            server: Some(server),
            ..self
        }
    }

    /// Set an optional auth token that the [Client] will use to authenticate 
    /// with the Interactsh server.
    /// 
    /// If this is not set, then no auth header will be sent to the 
    /// server.
    pub fn with_auth_token(self, auth_token: AuthToken) -> Self {
        Self {
            auth_token: Some(auth_token),
            ..self
        }
    }


    /// Set an optional proxy URL that the [Client] can use.
    /// 
    /// This can be set more than once; each new proxy URL will be added
    /// to a list of proxies that the [Client] will try.
    pub fn with_proxy(self, proxy: ClientProxy) -> Self {
        let proxies = match self.proxies {
            Some(mut proxies) => {
                proxies.push(proxy);
                Some(proxies)
            }
            None => Some(vec![proxy]),
        };

        Self {
            proxies,
            ..self
        }
    }


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
        Self {
            ssl_verify,
            ..self
        }
    }

    /// Sets whether or not the client should parse the logs
    /// or just return the raw logs.
    pub fn parse_logs(self, parse_logs: bool) -> Self {
        Self {
            parse_logs,
            ..self
        }
    }

    /// Builds an [UnregisteredClient].
    /// 
    /// The server must be set and the RSA key generated in order for
    /// this to succeed. If the build succeeds, the
    /// [register()](UnregisteredClient::register) function must be
    /// called on the returned [UnregisteredClient] to turn it into
    /// a usable [Client].
    pub fn build(self) -> Result<UnregisteredClient, ClientBuildError> {
        // Ensure rsa_key and server are set
        let rsa_key_size = self.rsa_key_size.ok_or(ClientBuildError::MissingRsaKeySize)?;
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

        reqwest_client_builder = match self.proxies {
            None => reqwest_client_builder,
            Some(proxies) => {
                let mut builder = reqwest_client_builder;

                for proxy in proxies.into_iter() {
                    builder = builder.proxy(proxy.into_reqwest_proxy()?);
                }

                builder
            }
        };

        let timeout = self.timeout.unwrap_or(Duration::from_secs(15));
        reqwest_client_builder = reqwest_client_builder.timeout(timeout);

        cfg_if::cfg_if! {
            if #[cfg(all(feature = "reqwest-rustls-tls", feature = "reqwest-native-tls"))] {
                reqwest_client_builder = reqwest_client_builder.use_rustls_tls();
            }
        }

        reqwest_client_builder = reqwest_client_builder.danger_accept_invalid_certs(self.ssl_verify);

        let reqwest_client = reqwest_client_builder.build()?;

        // Create the UnregisteredClient object
        let unreg_client = UnregisteredClient {
            rsa_key,
            server,
            sub_domain,
            correlation_id,
            auth_token: self.auth_token,
            secret_key: secret,
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
