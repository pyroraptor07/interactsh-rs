//! Error types for the [Client](crate::client) module.

use thiserror::Error;

use crate::client::client_helpers::Client;


// #[derive(Error, Debug)]
// #[error("Failed to create a proxy for reqwest")]
// pub struct ProxyConvertError {
//     #[from]
//     source: reqwest::Error,
// }


/// Inner error type for [ClientRegistrationError]
#[derive(Error, Debug)]
pub enum ClientRegistrationInnerError {
    #[error("Failed to send the request to the server")]
    RequestSendFailure {
        #[from]
        source: reqwest::Error,
    },

    #[error("Server returned an Unauthorized status code")]
    Unauthorized,

    #[error("Failed to register with the server - {status_code}: {server_msg}")]
    RegistrationFailure {
        server_msg: String,
        status_code: u16,
    },
}


/// Error returned during client registration or deregistration.
#[derive(Error, Debug)]
#[error("{error}")]
pub struct ClientRegistrationError<T> {
    pub error: ClientRegistrationInnerError,
    pub client: T,
}

impl<C> ClientRegistrationError<C>
where
    C: Client + Clone,
{
    pub(crate) fn new(client: C, inner_error: ClientRegistrationInnerError) -> Self {
        Self {
            error: inner_error,
            client,
        }
    }
}


/// The error type used by the [ClientBuilder](crate::client::ClientBuilder)
#[derive(Error, Debug)]
pub enum ClientBuildError {
    // #[error("Failed to set proxy")]
    // InvalidProxy {
    //     #[from]
    //     source: ProxyConvertError,
    // },
    #[error("Builder failed to generate the RSA private key")]
    RsaGen {
        #[from]
        source: super::RsaGenError,
    },

    #[error("RSA key size was not set")]
    MissingRsaKeySize,

    #[error("Interactsh server was not set")]
    MissingServer,

    #[error("Failed to extract the RSA public key")]
    PubKeyExtract {
        #[from]
        source: super::RsaGetPubKeyError,
    },

    #[error("Failed to encode the RSA public key")]
    PubKeyEncode {
        #[from]
        source: super::RsaEncodePubKeyError,
    },

    #[error("Failed to build the reqwest client")]
    ReqwestBuildFailed {
        #[from]
        source: reqwest::Error,
    },
}


/// The error type used by the [RegisteredClient](crate::client::RegisteredClient)
/// when polling the server
#[derive(Error, Debug)]
pub enum ClientPollError {
    #[error("Client failed to deregister with the Interactsh server")]
    Deregister,

    #[error("Client failed to poll the Interactsh server")]
    PollFailure {
        #[from]
        source: reqwest::Error,
    },

    #[error("Interactsh server returned error status - {status_code}: {server_msg}")]
    PollError {
        server_msg: String,
        status_code: u16,
    },

    #[error("Failed to decrypt the AES key")]
    AesKeyDecryptFailed {
        #[from]
        source: super::RsaDecryptError,
    },

    #[error("Failed to decrypt the received data")]
    DataDecryptFailed {
        #[from]
        source: super::AesDecryptError,
    },

    #[error("Base64 decoding failed")]
    Base64DecodeFailed {
        #[from]
        source: base64::DecodeError,
    },
}
