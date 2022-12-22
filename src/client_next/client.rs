use std::sync::Arc;
use std::time::Duration;

use futures::Stream;
use parking_lot::RwLock;
use secrecy::Secret;

use super::log_parsing::ParseLogs;
use super::log_stream::LogStream;
use crate::crypto::rsa::RSAPrivKey;
use crate::prelude::LogEntry;

pub(super) enum ClientStatus {
    Unregistered,
    Registered {
        sub_domain: String,
        correlation_id: String,
    },
}

pub enum LogPollResult {
    NoNewLogs,
    ReceivedLogs(Vec<LogEntry>),
    Error(snafu::Whatever),
}

pub struct CommInfo {
    pub(super) server_name: String,
    pub(super) auth_token: Option<Secret<String>>,
    pub(super) secret_key: Secret<String>,
    pub(super) encoded_pub_key: String,
}

pub struct InteractshClient {
    pub(super) status: Arc<RwLock<ClientStatus>>,
    pub(super) rsa_key: Arc<RSAPrivKey>,
    pub(super) server_comm_info: Arc<CommInfo>,
    pub(super) reqwest_client: Arc<reqwest::Client>,
    pub(super) parse_logs: bool,
}

impl InteractshClient {
    pub fn register(&self) -> Result<String, snafu::Whatever> {
        todo!()
    }

    pub fn deregister(&self) -> Result<(), snafu::Whatever> {
        todo!()
    }

    pub fn poll(&self) -> LogPollResult {
        todo!()
    }

    pub fn log_stream(&self, poll_period: Duration) -> impl Stream {
        LogStream::new(
            Arc::clone(&self.status),
            Arc::clone(&self.rsa_key),
            Arc::clone(&self.server_comm_info),
            Arc::clone(&self.reqwest_client),
            self.parse_logs,
            poll_period,
        )
    }

    pub fn get_interaction_fqdn(&self) -> Option<String> {
        todo!()
    }
}

impl ParseLogs for InteractshClient {}
