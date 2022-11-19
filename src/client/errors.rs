//! Error types for the [Client](crate::client) module.

use snafu::prelude::*;
use snafu::Backtrace;

use super::http_utils::Client;
use crate::crypto::errors::CryptoError;


/// Inner error type for [ClientRegistrationError]
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum RegistrationError {
    #[snafu(display("Failed to send the request to the server"))]
    RequestSendFailure { source: reqwest::Error },

    #[snafu(display("Server returned an Unauthorized status code"))]
    Unauthorized { backtrace: Backtrace },

    #[snafu(display(
        "Failed to register or deregister with the server - {status_code}: {server_msg}"
    ))]
    RegistrationFailure {
        server_msg: String,
        status_code: u16,
        backtrace: Backtrace,
    },
}

/// Error returned during client registration or deregistration
///
/// This is a wrapper error. See [RegistrationError] for the inner error types.
#[derive(Debug, Snafu)]
#[snafu(
    module,
    context(suffix(false)),
    visibility(pub(crate)),
    display("Failure occured during registration/deregistration")
)]
pub struct ClientRegistrationError<C: Client + Clone> {
    #[snafu(source)]
    pub error: RegistrationError,
    pub client: C,
}


/// Errors returned by the [ClientBuilder](crate::client::ClientBuilder)
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum ClientBuildError {
    #[snafu(display("Builder failed to generate the RSA private key"))]
    RsaGen { source: CryptoError },

    #[snafu(display("RSA key size was not set"))]
    MissingRsaKeySize { backtrace: Backtrace },

    #[snafu(display("Interactsh server was not set"))]
    MissingServer { backtrace: Backtrace },

    #[snafu(display("Failed to extract the RSA public key"))]
    PubKeyExtract { source: CryptoError },

    #[snafu(display("Failed to encode the RSA public key"))]
    PubKeyEncode { source: CryptoError },

    #[snafu(display("Failed to build the reqwest client"))]
    ReqwestBuildFailed { source: reqwest::Error },
}


/// Errors returned by the [RegisteredClient](crate::client::RegisteredClient) when polling the server
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum ClientPollError {
    #[snafu(display("Client failed to poll the Interactsh server"))]
    PollFailure { source: reqwest::Error },

    #[snafu(display("Interactsh server returned error status - {status_code}: {server_msg}"))]
    PollErrorStatus {
        server_msg: String,
        status_code: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("Server response is not valid JSON"))]
    ResponseJsonParseFailed { source: reqwest::Error },

    #[snafu(display("Failed to decrypt the AES key"))]
    AesKeyDecryptFailed { source: CryptoError },

    #[snafu(display("Failed to decrypt the received data"))]
    DataDecryptFailed { source: CryptoError },

    #[snafu(display("Base64 decoding failed"))]
    Base64DecodeFailed { source: base64::DecodeError },
}
