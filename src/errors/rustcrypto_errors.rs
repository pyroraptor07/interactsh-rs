//! The error types used by the [crypto](crate::crypto) module when the
//! `rustcrypto` feature is enabled

use thiserror::Error;


#[derive(Error, Debug)]
pub enum AesDecryptError {
    #[error("Unable to decrypt data with provided AES key")]
    DecryptFailure {
        #[from]
        source: inout::NotEqualError,
    },

    #[error("Failed to decode the data using base 64 encoding")]
    DecodeFailure {
        #[from]
        source: base64::DecodeError,
    },
}


#[derive(Error, Debug)]
#[error("Failed to generate the RSA private key")]
pub struct RsaGenError {
    #[from]
    source: rsa::errors::Error,
}


#[derive(Error, Debug)]
#[error("Failed to decrypt the data with the provided RSA private key")]
pub struct RsaDecryptError {
    #[from]
    source: rsa::errors::Error,
}


#[derive(Error, Debug)]
#[error("Failed to extract the RSA public key from the RSA private key")]
pub struct RsaGetPubKeyError;


#[derive(Error, Debug)]
#[error("Failed to encode the RSA public key as a base 64 string")]
pub struct RsaEncodePubKeyError {
    #[from]
    source: rsa::pkcs8::spki::Error,
}
