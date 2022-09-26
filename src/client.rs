//! Defines the client struct used for communicating with the Interactsh servers.

use std::fmt::Display;
use std::time::Duration;
use std::vec;

use rand::seq::SliceRandom;
use reqwest::StatusCode;
use uuid::Uuid;
use serde_json::json;
use svix_ksuid::*;

use crate::crypto::aes;
use crate::crypto::rsa::RSAPrivKey;
use crate::crypto::hash::{Sha2HashAlgo, Sha2HashAlgoType};
use crate::errors::{ClientError, ClientBuildError, ProxyConvertError, ClientRegistrationError, ClientRegistrationInnerError};

/// The default list of servers provided by the Interactsh team
const DEFAULT_INTERACTSH_SERVERS: &[&str] = &[
    "oast.pro",
    "oast.live",
    "oast.site",
    "oast.online",
    "oast.fun",
    "oast.me",
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


#[derive(Debug)]
pub enum ProxyType {
    Http,
    Https,
    #[cfg(feature = "reqwest-socks-proxy")]
    SocksV5,
}

#[cfg(not(feature = "reqwest-socks-proxy"))]
impl Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => write!(f, "http://"),
            Self::Https => write!(f, "https://"),
        }
    }
}

#[cfg(feature = "reqwest-socks-proxy")]
impl Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => write!(f, "http://"),
            Self::Https => write!(f, "https://"),
            Self::SocksV5 => write!(f, "socks5://"),
        }
    }
}


#[derive(Debug)]
pub struct ClientProxy {
    server: String,
    proxy_type: ProxyType,
    port: Option<u16>,
}

impl ClientProxy {
    pub fn new(server: String, proxy_type: ProxyType, port: Option<u16>) -> Self {
        Self { server, proxy_type, port }
    }

    fn into_reqwest_proxy(self) -> Result<reqwest::Proxy, ProxyConvertError> {
        let mut full_url = format!("{}{}", self.proxy_type, self.server);
        if let Some(port) = self.port {
            full_url.push_str(format!(":{}", port).as_str());
        }

        let proxy = reqwest::Proxy::all(full_url)?;

        Ok(proxy)
    }
}

/// Builder for the [Client] struct
pub struct ClientBuilder {
    rsa_key: Option<RSAPrivKey>,
    server: Option<String>,
    auth_token: Option<AuthToken>,
    proxies: Option<Vec<ClientProxy>>,
    timeout: Option<Duration>,
    ssl_verify: bool,
}

impl ClientBuilder {
    /// Create a new builder with no options defined.
    pub fn new() -> Self {
        Self {
            rsa_key: None,
            server: None,
            auth_token: None,
            proxies: None,
            timeout: None,
            ssl_verify: false,
        }
    }

    /// Create a new builder with the default options.
    /// 
    /// This will create a builder with a 2048 bit RSA key and server randomly picked from the
    /// [list of default servers](https://github.com/projectdiscovery/interactsh#using-self-hosted-server) 
    /// provided and maintained by the Interactsh team. This will also set the timeout
    /// to 15 seconds and SSL verification to false.
    pub fn default() -> Result<Self, ClientBuildError> {
        let rsa_key = RSAPrivKey::generate(2048)?;
        let server = *DEFAULT_INTERACTSH_SERVERS.choose(&mut rand::thread_rng()).expect("Unable to pick a server from the default list!");

        let new_builder = Self {
            rsa_key: Some(rsa_key),
            server: Some(server.to_string()),
            auth_token: None,
            proxies: None,
            timeout: Some(Duration::from_secs(15)),
            ssl_verify: false,
        };

        Ok(new_builder)
    }

    /// Generate a new RSA private key for the [Client] to use of the bit size given.
    pub fn gen_rsa_key(self, num_bits: usize) -> Result<Self, ClientBuildError> {
        let rsa_key = RSAPrivKey::generate(num_bits)?;

        let new_builder = Self {
            rsa_key: Some(rsa_key),
            ..self
        };

        Ok(new_builder)
    }

    /// Set a predefined RSA private key for the [Client] to use.
    pub fn with_rsa_key(self, rsa_key: RSAPrivKey) -> Self {
        Self {
            rsa_key: Some(rsa_key),
            ..self
        }
    }

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

    /// Builds an [UnregisteredClient].
    /// 
    /// The server must be set and the RSA key generated in order for
    /// this to succeed. If the build succeeds, the
    /// [register()](UnregisteredClient::register) function must be
    /// called on the returned [UnregisteredClient] to turn it into
    /// a usable [Client].
    pub fn build(self) -> Result<UnregisteredClient, ClientBuildError> {
        // Ensure rsa_key and server are set
        let rsa_key = self.rsa_key.ok_or(ClientBuildError::MissingRsaKey)?;
        let server = self.server.ok_or(ClientBuildError::MissingServer)?;

        // Get the other values needed
        let pubkey = rsa_key.get_pub_key()?;
        let secret = Uuid::new_v4().to_string();
        let encoded_pub_key = pubkey.b64_encode()?;
        let sub_domain = Ksuid::new(None, None).to_string();

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
        };

        Ok(unreg_client)
    }
}

#[derive(Debug, Clone)]
pub struct UnregisteredClient {
    rsa_key: RSAPrivKey,
    server: String,
    sub_domain: String,
    correlation_id: String,
    auth_token: Option<AuthToken>,
    secret_key: String,
    encoded_pub_key: String,
    reqwest_client: reqwest::Client,
}

impl UnregisteredClient {
    pub async fn register(self) -> Result<Client, ClientRegistrationError> {
        let data = json!({
            "public-key": self.encoded_pub_key.as_str(),
            "secret-key": self.secret_key.as_str(),
            "correlation-id": self.correlation_id.as_str()
        });

        let register_url = format!("https://{}/register", self.server);
        let mut post_request = self.reqwest_client.post(register_url);

        post_request = match &self.auth_token {
            Some(AuthToken::SimpleAuth(token)) => post_request.header("Authorization", token),
            Some(AuthToken::BearerAuth(token)) => post_request.bearer_auth(token),
            None => post_request,
        };

        let register_response = post_request
            .json(&data)
            .send()
            .await
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


/// Wrapper struct around a log entry.
/// 
/// Returned when [Client] polls the server and receives
/// a new log.
#[derive(Debug)]
pub struct LogEntry {
    pub log: String,
}


#[derive(serde::Deserialize)]
struct PollResponse {
    aes_key: String,

    #[serde(rename(deserialize = "data"))]
    data_list: Vec<String>,
}


/// The primary struct used to communicate with an
/// Interactsh server.
pub struct Client {
    rsa_key: RSAPrivKey,
    server: String,
    sub_domain: String,
    correlation_id: String,
    auth_token: Option<AuthToken>,
    secret_key: String,
    encoded_pub_key: String,
    reqwest_client: reqwest::Client,
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

        let get_response = get_request.send().await?;
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
        
        let mut results = Vec::new();
        for data in response_body.data_list.iter() {
            let decrypted_data = self.decrypt_data(&response_body.aes_key, data)?;
            let log_entry = LogEntry { log: decrypted_data };
            results.push(log_entry);
        }

        Ok(results)
    }

    fn decrypt_data(&self, aes_key: &String, encrypted_data: &String) -> Result<String, ClientError> {
        let hash_algorithm = Sha2HashAlgo::new(Sha2HashAlgoType::Sha256);
        let aes_plain_key = self.rsa_key.decrypt_data(hash_algorithm, aes_key.as_bytes())?;

        let decrypted_data = aes::decrypt_data(&aes_plain_key, encrypted_data.as_bytes())?;

        let decrypted_string = String::from_utf8_lossy(&decrypted_data);

        Ok(decrypted_string.into())
    }
}
