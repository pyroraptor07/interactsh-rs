//! The error types used by the [crypto](crate::crypto) module when the
//! `openssl` feature is enabled (and `rustcrypto` is not enabled)

#[cfg(feature = "nightly")]
use std::backtrace::Backtrace;

use thiserror::Error;


cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        #[derive(Error, Debug)]
        pub enum AesDecryptError {
            #[error("Unable to decrypt data with provided AES key")]
            DecryptFailure {
                #[from]
                source: openssl::error::ErrorStack,
                backtrace: Backtrace,
            },
        
            #[error("Failed to decode the data using base 64 encoding")]
            DecodeFailure {
                #[from]
                source: base64::DecodeError,
                backtrace: Backtrace,
            }
        }
    } else {
        #[derive(Error, Debug)]
        pub enum AesDecryptError {
            #[error("Unable to decrypt data with provided AES key")]
            DecryptFailure {
                #[from]
                source: openssl::error::ErrorStack,
            },
        
            #[error("Failed to decode the data using base 64 encoding")]
            DecodeFailure {
                #[from]
                source: base64::DecodeError,
            }
        }
    }
}


cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        #[derive(Error, Debug)]
        pub enum RsaGenInnerError {
            #[error("Requested RSA key bit size is too large")]
            BitSize,
        
            #[error("OpenSSL failed to generate the RSA private key")]
            OpenSsl {
                #[from]
                source: openssl::error::ErrorStack,
                backtrace: Backtrace,
            },
        }
    } else {
        #[derive(Error, Debug)]
        pub enum RsaGenInnerError {
            #[error("Requested RSA key bit size is too large")]
            BitSize,
        
            #[error("OpenSSL failed to generate the RSA private key")]
            OpenSsl {
                #[from]
                source: openssl::error::ErrorStack,
            },
        }
    }
}


cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        #[derive(Error, Debug)]
        #[error("Failed to generate the RSA private key")]
        pub struct RsaGenError {
            #[from]
            source: RsaGenInnerError,
            backtrace: Backtrace,
        }
    } else {
        #[derive(Error, Debug)]
        #[error("Failed to generate the RSA private key")]
        pub struct RsaGenError {
            #[from]
            source: RsaGenInnerError,
        }
    }
}


cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        #[derive(Error, Debug)]
        #[error("Failed to decrypt the data with the provided RSA private key")]
        pub struct RsaDecryptError {
            #[from]
            source: openssl::error::ErrorStack,
            backtrace: Backtrace,
        }
    } else {
        #[derive(Error, Debug)]
        #[error("Failed to decrypt the data with the provided RSA private key")]
        pub struct RsaDecryptError {
            #[from]
            source: openssl::error::ErrorStack,
        }
    }
}


cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        #[derive(Error, Debug)]
        #[error("Failed to extract the RSA public key from the RSA private key")]
        pub struct RsaGetPubKeyError {
            #[from]
            source: openssl::error::ErrorStack,
            backtrace: Backtrace,
        }
    } else {
        #[derive(Error, Debug)]
        #[error("Failed to extract the RSA public key from the RSA private key")]
        pub struct RsaGetPubKeyError {
            #[from]
            source: openssl::error::ErrorStack,
        }
    }
}


cfg_if::cfg_if! {
    if #[cfg(feature = "nightly")] {
        #[derive(Error, Debug)]
        #[error("Failed to encode the RSA public key as a base 64 string")]
        pub struct RsaEncodePubKeyError {
            #[from]
            source: openssl::error::ErrorStack,
            backtrace: Backtrace,
        }
    } else {
        #[derive(Error, Debug)]
        #[error("Failed to encode the RSA public key as a base 64 string")]
        pub struct RsaEncodePubKeyError {
            #[from]
            source: openssl::error::ErrorStack,
        }
    }
}