use std::sync::Arc;

use parking_lot::RwLock;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use secrecy::Secret;
use snafu::Whatever;

#[cfg(feature = "log-stream")]
use self::log_stream::*;
use crate::client_shared::http_utils::PollResponse;
use crate::crypto::rsa::RSAPrivKey;
use crate::interaction_log::LogEntry;

#[cfg(feature = "log-stream")]
mod log_stream {
    pub use std::time::Duration;

    pub use async_io::Timer;
    pub use async_stream::stream;
    pub use futures_util::{StreamExt, TryStream};
    pub use snafu::ResultExt;
}

pub(crate) struct CorrelationConfig {
    pub subdomain_length: usize,
    pub correlation_id_length: usize,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            subdomain_length: 33,
            correlation_id_length: 20,
        }
    }
}

pub(crate) struct CorrelationData {
    subdomain: String,
    correlation_id: String,
}

impl CorrelationData {
    fn generate_data(config: &CorrelationConfig) -> Self {
        let max_length = config.subdomain_length.max(config.correlation_id_length);
        let mut subdomain = Alphanumeric
            .sample_string(&mut thread_rng(), max_length)
            .to_ascii_lowercase();

        let mut correlation_id = subdomain.clone();

        subdomain.truncate(config.subdomain_length);
        correlation_id.truncate(config.correlation_id_length);

        Self {
            subdomain,
            correlation_id,
        }
    }
}

impl Default for CorrelationData {
    fn default() -> Self {
        Self::generate_data(&CorrelationConfig::default())
    }
}

impl From<CorrelationData> for ClientStatus {
    fn from(data: CorrelationData) -> Self {
        ClientStatus::Registered {
            subdomain: data.subdomain,
            correlation_id: data.correlation_id,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ClientStatus {
    Unregistered,
    Registered {
        subdomain: String,
        correlation_id: String,
    },
}

pub struct ServerComm {
    pub(super) server_name: String,
    pub(super) auth_token: Option<Secret<String>>,
    pub(super) secret_key: Secret<String>,
    pub(super) encoded_pub_key: String,
    pub(super) reqwest_client: Arc<reqwest::Client>,
    pub(super) correlation_config: Option<CorrelationConfig>,
    pub(super) status: ClientStatus,
}

impl ServerComm {
    pub(crate) fn get_interaction_fqdn(&self) -> Option<String> {
        match &self.status {
            ClientStatus::Unregistered => None,
            ClientStatus::Registered { subdomain, .. } => {
                Some(format!("{}.{}", subdomain, self.server_name))
            }
        }
    }

    pub(crate) async fn register(&mut self) -> Result<String, Whatever> {
        todo!()
    }

    pub(crate) async fn deregister(&mut self) -> Result<(), Whatever> {
        todo!()
    }

    pub(crate) async fn force_deregister(&mut self) {
        self.deregister().await.ok();
        self.status = ClientStatus::Unregistered;
    }

    pub(crate) async fn poll(&self) -> Result<PollResponse, reqwest::Error> {
        todo!()
    }
}

pub struct InteractshClient {
    pub(super) rsa_key: Arc<RSAPrivKey>,
    pub(super) server_comm: Arc<RwLock<ServerComm>>,
    pub(super) parse_logs: bool,
}

impl InteractshClient {
    pub fn get_interaction_fqdn(&self) -> Option<String> {
        let comm = self.server_comm.read();
        comm.get_interaction_fqdn()
    }

    pub async fn register(&self) -> Result<String, Whatever> {
        let mut comm = self.server_comm.write();
        comm.register().await
    }

    pub async fn deregister(&self) -> Result<(), Whatever> {
        let mut comm = self.server_comm.write();
        comm.deregister().await
    }

    pub async fn force_deregister(&self) {
        let mut comm = self.server_comm.write();
        comm.force_deregister().await;
    }

    pub async fn poll(&self) -> Result<Option<Vec<LogEntry>>, Whatever> {
        let comm = self.server_comm.read();
        let response = comm.poll().await.whatever_context("Poll failed");
        drop(comm);

        match response {
            Ok(response) => {
                match decrypt_logs(response, self.rsa_key.as_ref(), self.parse_logs) {
                    Ok(new_logs) => {
                        if new_logs.is_empty() {
                            Ok(None)
                        } else {
                            Ok(Some(new_logs))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    #[cfg(feature = "log-stream")]
    pub fn log_stream(
        &self,
        poll_period: Duration,
    ) -> impl TryStream<Ok = LogEntry, Error = Whatever> {
        let server_comm = Arc::clone(&self.server_comm);
        let rsa_key = Arc::clone(&self.rsa_key);
        let parse_logs = self.parse_logs;

        stream! {
            let comm = server_comm.read();
            if comm.status == ClientStatus::Unregistered {
                return ();
            }
            drop(comm);
            let mut timer = Timer::interval(poll_period);

            loop {
                timer.next().await;

                let comm = server_comm.read();
                if comm.status == ClientStatus::Unregistered {
                    break;
                }

                let response = comm.poll().await.whatever_context("Poll failed");
                drop(comm);

                match response {
                    Ok(response) => {
                        match decrypt_logs(response, rsa_key.as_ref(), parse_logs) {
                            Ok(new_logs) => {
                                if new_logs.is_empty() {
                                    continue;
                                }

                                for log in new_logs {
                                    yield Ok(log);
                                }
                            },
                            Err(e) => yield Err(e),
                        }
                    },
                    Err(e) => yield Err(e),
                }
            }
        }
    }
}

fn decrypt_logs(
    response: PollResponse,
    rsa_key: &RSAPrivKey,
    parse_logs: bool,
) -> Result<Vec<LogEntry>, Whatever> {
    todo!()
}
