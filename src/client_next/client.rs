use std::sync::Arc;

use parking_lot::RwLock;
use secrecy::Secret;
use snafu::Whatever;

#[cfg(feature = "log-stream")]
use self::log_stream::*;
use super::correlation::CorrelationConfig;
use crate::client_shared::http_utils::PollResponse;
use crate::crypto::rsa::RSAPrivKey;
use crate::interaction_log::LogEntry;

#[cfg(feature = "log-stream")]
mod log_stream {
    pub use std::time::Duration;

    pub use async_io::Timer;
    pub use async_stream::stream;
    pub use futures_util::{Stream, StreamExt, TryStream};
    pub use snafu::ResultExt;
}


#[derive(PartialEq, Eq)]
pub enum ClientStatus {
    Unregistered,
    Registered {
        subdomain: String,
        correlation_id: String,
    },
}

pub enum LogPollResult {
    PollFail(Whatever),
    DecryptFail(Whatever),
    NoNewLogs,
    ReceivedNewLog(LogEntry),
}

pub struct ServerComm {
    pub(crate) server_name: String,
    pub(crate) auth_token: Option<Secret<String>>,
    pub(crate) secret_key: Secret<String>,
    pub(crate) encoded_pub_key: String,
    pub(crate) reqwest_client: Arc<reqwest::Client>,
    pub(crate) correlation_config: Option<CorrelationConfig>,
    pub(crate) status: ClientStatus,
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
    pub(crate) rsa_key: Arc<RSAPrivKey>,
    pub(crate) server_comm: Arc<RwLock<ServerComm>>,
    pub(crate) parse_logs: bool,
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
        let response = {
            let comm = self.server_comm.read();
            comm.poll().await.whatever_context("context")
        };

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

    /// Returns a [Stream](futures_util::Stream) that will poll the Interactsh server as long
    /// as the client remains registered, and will run the provided map function on each
    /// result. For any Some(R) value return from the map function, the stream will yield back
    /// the contained value. If more than one log is returned from a single poll, the map
    /// function will run on each log. The provided map function should accept and process
    /// a [LogPollResult].
    #[cfg(feature = "log-stream")]
    pub fn log_stream_map_filter<F, R>(
        &self,
        poll_period: Duration,
        map_fn: F,
    ) -> impl Stream<Item = R>
    where
        F: Fn(LogPollResult) -> Option<R>,
    {
        let server_comm = Arc::clone(&self.server_comm);
        let rsa_key = Arc::clone(&self.rsa_key);
        let parse_logs = self.parse_logs;

        stream! {
            {
                let comm = server_comm.read();
                if comm.status == ClientStatus::Unregistered {
                    return ();
                }
            }

            let mut timer = Timer::interval(poll_period);

            'poll_loop: loop {
                timer.next().await;

                let response = {
                    let comm = server_comm.read();
                    if comm.status == ClientStatus::Unregistered {
                        break 'poll_loop;
                    }

                    comm.poll().await.whatever_context("Poll failed")
                };

                match response {
                    Ok(response) => {
                        match decrypt_logs(response, rsa_key.as_ref(), parse_logs) {
                            Ok(new_logs) => {
                                if new_logs.is_empty() {
                                    if let Some(return_val) = map_fn(LogPollResult::NoNewLogs) {
                                        yield return_val;
                                    }
                                    continue 'poll_loop;
                                }

                                for log in new_logs {
                                    if let Some(return_val) = map_fn(LogPollResult::ReceivedNewLog(log)) {
                                        yield return_val;
                                    }
                                }
                            },
                            Err(e) => {
                                if let Some(return_val) = map_fn(LogPollResult::DecryptFail(e)) {
                                    yield return_val;
                                }
                            },
                        }
                    },
                    Err(e) => {
                        if let Some(return_val) = map_fn(LogPollResult::PollFail(e)) {
                            yield return_val;
                        }
                    },
                }
            }
        }
    }

    /// Convenience wrapper around [log_stream_map_filter()] that ignores empty poll responses
    /// and returns the errors and decrypted LogEntry objects wrapped in a Result type.
    #[cfg(feature = "log-stream")]
    pub fn log_stream(
        &self,
        poll_period: Duration,
    ) -> impl TryStream<Ok = LogEntry, Error = Whatever> {
        self.log_stream_map_filter(poll_period, |res| {
            match res {
                LogPollResult::NoNewLogs => None,
                LogPollResult::ReceivedNewLog(log) => Some(Ok(log)),
                LogPollResult::PollFail(e) | LogPollResult::DecryptFail(e) => Some(Err(e)),
            }
        })
    }
}

fn decrypt_logs(
    response: PollResponse,
    rsa_key: &RSAPrivKey,
    parse_logs: bool,
) -> Result<Vec<LogEntry>, Whatever> {
    todo!()
}
