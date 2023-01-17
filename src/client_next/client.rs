use std::sync::Arc;

use parking_lot::RwLock;
use secrecy::Secret;
use snafu::Whatever;

#[cfg(feature = "log-stream")]
pub use self::log_stream::*;
use crate::client_shared::http_utils::PollResponse;
use crate::crypto::rsa::RSAPrivKey;
use crate::interaction_log::LogEntry;

#[derive(PartialEq, Eq)]
pub enum ClientStatus {
    Unregistered,
    Registered {
        sub_domain: String,
        correlation_id: String,
    },
}

pub struct ServerComm {
    pub(super) server_name: String,
    pub(super) auth_token: Option<Secret<String>>,
    pub(super) secret_key: Secret<String>,
    pub(super) encoded_pub_key: String,
    pub(super) reqwest_client: Arc<reqwest::Client>,
    pub(super) status: ClientStatus,
}

impl ServerComm {
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

    pub async fn poll(&self) -> Result<Vec<LogEntry>, Whatever> {
        let comm = self.server_comm.read();
        let response = comm.poll();

        todo!()
    }

    pub fn get_interaction_fqdn(&self) -> Option<String> {
        todo!()
    }
}

fn decrypt_logs(
    response: PollResponse,
    rsa_key: &RSAPrivKey,
    parse_logs: bool,
) -> Result<Vec<LogEntry>, Whatever> {
    todo!()
}

#[cfg(feature = "log-stream")]
mod log_stream {
    use std::sync::Arc;
    use std::time::Duration;

    use async_io::Timer;
    use async_stream::stream;
    use futures_util::{StreamExt, TryStream};
    use snafu::{ResultExt, Whatever};

    use super::{decrypt_logs, ClientStatus, InteractshClient};
    use crate::interaction_log::LogEntry;

    impl InteractshClient {
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
}
