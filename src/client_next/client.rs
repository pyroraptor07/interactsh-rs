use std::sync::Arc;

use parking_lot::RwLock;
use smallvec::SmallVec;
use snafu::Whatever;

#[cfg(feature = "log-stream")]
use self::log_stream::*;
use crate::client_shared::log_decrypt::decrypt_logs;
use crate::client_shared::server_comm::{ClientStatus, ServerComm};
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


#[cfg(feature = "log-stream")]
pub enum LogPollResult {
    Error(Whatever),
    NoNewLogs,
    ReceivedNewLog(LogEntry),
}


#[derive(Debug)]
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
    /// result. For any Some(R) value returned from the map function, the stream will yield back
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
                            Err(e) => return_vals.push(map_fn(LogPollResult::Error(e))),
                        }
                    },
                    Err(e) => return_vals.push(map_fn(LogPollResult::Error(e))),
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
    ) -> impl Stream<Item = Result<LogEntry, Whatever>> {
        self.log_stream_map_filter(poll_period, |res| {
            match res {
                LogPollResult::NoNewLogs => None,
                LogPollResult::ReceivedNewLog(log) => Some(Ok(log)),
                LogPollResult::Error(e) => Some(Err(e)),
            }
        })
    }
}


#[cfg(test)]
mod tests {
    // use std::time::Duration;

    // use futures_util::{pin_mut, StreamExt};

    // // use super::*;
    // use crate::client_next::*;

    // #[test]
    // fn log_stream_works() {
    //     let future = async {
    //         color_eyre::install().ok();

    //         let client = ClientBuilder::default().build().unwrap();
    //         client.register().await.unwrap();

    //         let interaction_url = format!("https://{}", client.get_interaction_fqdn().unwrap());
    //         reqwest::get(interaction_url).await.unwrap();

    //         let log_stream = client.log_stream(Duration::from_secs(5));
    //         pin_mut!(log_stream);
    //         let poll_result = log_stream.next().await;

    //         assert!(poll_result.is_some());
    //         client.force_deregister().await;
    //     };

    //     smol::block_on(future);
    // }
}
