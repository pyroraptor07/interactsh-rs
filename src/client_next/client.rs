use parking_lot::RwLock;
use secrecy::Secret;

use crate::crypto::rsa::RSAPrivKey;
use crate::prelude::LogEntry;

pub(super) enum ClientStatus {
    Unregistered,
    Registered {
        sub_domain: String,
        correlation_id: String,
    },
}

#[derive(Clone)]
pub struct ClientHandle {
    // some async channel here
}

impl ClientHandle {
    pub fn deregister_client(&self) -> Result<(), snafu::Whatever> {
        todo!()
    }
}

pub(super) enum ClientHandleState {
    None,
    Created {
        handle: ClientHandle,
        // some async channel here
    },
}

pub struct InteractshClient {
    pub(super) status: RwLock<ClientStatus>,
    pub(super) rsa_key: RSAPrivKey,
    pub(super) server: String,
    pub(super) auth_token: Option<Secret<String>>,
    pub(super) secret_key: Secret<String>,
    pub(super) encoded_pub_key: String,
    pub(super) reqwest_client: reqwest::Client,
    pub(super) parse_logs: bool,
    pub(super) handle_state: RwLock<ClientHandleState>,
}

impl InteractshClient {
    pub fn register(&self) -> Result<String, snafu::Whatever> {
        todo!()
    }

    pub fn deregister(&self) -> Result<(), snafu::Whatever> {
        todo!()
    }

    pub fn poll(&self) -> Result<Option<Vec<LogEntry>>, snafu::Whatever> {
        todo!()
    }

    pub fn get_interaction_fqdn(&self) -> Option<String> {
        todo!()
    }
}
