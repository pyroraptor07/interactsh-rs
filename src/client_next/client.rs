use std::sync::Arc;

use parking_lot::RwLock;
use smallvec::SmallVec;
use snafu::Whatever;

#[cfg(feature = "log-stream")]
use self::log_stream::*;
use crate::client_shared::http_utils::{PollResponse, ServerComm};
use crate::crypto::aes;
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


pub enum LogPollResult {
    PollFail(Whatever),
    DecryptFail(Whatever),
    NoNewLogs,
    ReceivedNewLog(LogEntry),
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

    pub async fn register(&self) -> Result<(), Whatever> {
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
            comm.poll().await
        };

        match response {
            Ok(response) => decrypt_logs(response, self.rsa_key.as_ref(), self.parse_logs),
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
        use crate::client_shared::http_utils::ClientStatus;

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

                let mut return_vals = SmallVec::<[Option<R>; 1]>::new();
                match response {
                    Ok(response) => {
                        match decrypt_logs(response, rsa_key.as_ref(), parse_logs) {
                            Ok(Some(new_logs)) => {
                                new_logs
                                    .into_iter()
                                    .map(|log| map_fn(LogPollResult::ReceivedNewLog(log)))
                                    .for_each(|val| return_vals.push(val));
                            },
                            Ok(None) => return_vals.push(map_fn(LogPollResult::NoNewLogs)),
                            Err(e) => return_vals.push(map_fn(LogPollResult::DecryptFail(e))),
                        }
                    },
                    Err(e) => return_vals.push(map_fn(LogPollResult::PollFail(e))),
                }

                let return_vals = return_vals.into_iter().filter_map(|val| val);

                for val in return_vals {
                    yield val;
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
) -> Result<Option<Vec<LogEntry>>, Whatever> {
    // Return None if empty
    let response_body_data = match response.data_list {
        Some(data) => {
            if data.is_empty() {
                return Ok(None);
            } else {
                data
            }
        }
        None => return Ok(None),
    };

    // Decode and decrypt AES key
    let aes_key_decoded =
        base64::decode(&response.aes_key).whatever_context("AES key base 64 decode failed")?;
    let aes_plain_key = rsa_key
        .decrypt_data(&aes_key_decoded)
        .whatever_context("Failed to decrypt aes key")?;

    // Decode and decrypt logs
    let mut decrypted_logs = Vec::new();
    for encoded_data in response_body_data.iter() {
        let encrypted_data =
            base64::decode(encoded_data).whatever_context("Data base 64 decode failed")?;

        let decrypted_data = aes::decrypt_data(&aes_plain_key, &encrypted_data)
            .whatever_context("Failed to decrypt data")?;

        let decrypted_string = String::from_utf8_lossy(&decrypted_data);

        let log_entry = LogEntry::new_log_entry(&decrypted_string, parse_logs);

        decrypted_logs.push(log_entry);
    }

    Ok(Some(decrypted_logs))
}
