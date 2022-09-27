//! The error types used by the [Client](crate::client::Client) and 
//! [ClientBuilder](crate::client::ClientBuilder) structs

#[cfg(feature = "nightly")]
use std::backtrace::Backtrace;

use thiserror::Error;


#[derive(Error, Debug)]
#[error("Failed to create a proxy for reqwest")]
pub struct ProxyConvertError {
    #[from]
    source: reqwest::Error,
    #[cfg(feature = "nightly")]
    backtrace: Backtrace,
}


#[derive(Error, Debug)]
pub enum ClientRegistrationInnerError {
    #[error("Failed to send the request to the server")]
    RequestSendFailure {
        #[from]
        source: reqwest::Error,
        #[cfg(feature = "nightly")]
        backtrace: Backtrace,
    },

    #[error("Server returned an Unauthorized status code")]
    Unauthorized,

    #[error("Failed to register with the server - {status_code}: {server_msg}")]
    RegistrationFailure {
        server_msg: String,
        status_code: u16,
    },
}

#[derive(Error, Debug)]
#[error("{error}")]
pub struct ClientRegistrationError {
    pub error: ClientRegistrationInnerError,
    pub unregistered_client: crate::client::UnregisteredClient,
}


/// The error type used by the [ClientBuilder](crate::client::ClientBuilder)
#[derive(Error, Debug)]
pub enum ClientBuildError {
    #[error("Failed to set proxy")]
    InvalidProxy {
        #[from]
        source: ProxyConvertError,
        #[cfg(feature = "nightly")]
        backtrace: Backtrace,
    },

    #[error("Builder failed to generate the RSA private key")]
    RsaGen {
        #[from]
        source: super::RsaGenError,
        #[cfg(feature = "nightly")]
        backtrace: Backtrace,
    },

    #[error("RSA key size was not set")]
    MissingRsaKeySize,

    #[error("Interactsh server was not set")]
    MissingServer,

    #[error("Failed to extract the RSA public key")]
    PubKeyExtract {
        #[from]
        source: super::RsaGetPubKeyError,
        #[cfg(feature = "nightly")]
        backtrace: Backtrace,
    },

    #[error("Failed to encode the RSA public key")]
    PubKeyEncode {
        #[from]
        source: super::RsaEncodePubKeyError,
        #[cfg(feature = "nightly")]
        backtrace: Backtrace,
    },

    #[error("Failed to build the reqwest client")]
    ReqwestBuildFailed {
        #[from]
        source: reqwest::Error,
        #[cfg(feature = "nightly")]
        backtrace: Backtrace,
    },
}


/// The error type used by the [Client](crate::client::Client)
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Client failed to deregister with the Interactsh server")]
    Deregister,

    #[error("Client failed to poll the Interactsh server")]
    PollFailure {
        #[from]
        source: reqwest::Error,
        #[cfg(feature = "nightly")]
        backtrace: Backtrace,
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
        #[cfg(feature = "nightly")]
        backtrace: Backtrace,
    },

    #[error("Failed to decrypt the received data")]
    DataDecryptFailed {
        #[from]
        source: super::AesDecryptError,
        #[cfg(feature = "nightly")]
        backtrace: Backtrace,
    }
}
