use snafu::prelude::*;
use snafu::{FromString, Whatever};

use crate::error::CryptoError::{AesDecryptError, AesDecryptorBuildError};

/// The main error type that should be returned by any implementer of the
/// [AesDecryptor](crate::aes::AesDecryptor) and XXX traits
///
/// The [ToCryptoError] trait provides helper functions for converting other errors
/// into CryptoError variants and has been implemented for any result types with string
/// errors as well as errors that implement the [Error](std::error::Error) trait.
#[derive(Debug, Clone, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum CryptoError {
    #[snafu(display("Aes decryption error: {source}"))]
    AesDecryptError { source: Box<dyn std::error::Error> },

    #[snafu(display("Aes decryptor build error: {source}"))]
    AesDecryptorBuildError { source: Box<dyn std::error::Error> },
}


pub trait ToCryptoError<T> {
    fn aes_decrypt_error(self) -> Result<T, CryptoError>;
    fn aes_decryptor_build_error(self) -> Result<T, CryptoError>;
}

impl<T, E> ToCryptoError<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn aes_decrypt_error(self) -> Result<T, CryptoError> {
        self.context(AesDecryptError)
    }

    fn aes_decryptor_build_error(self) -> Result<T, CryptoError> {
        self.context(AesDecryptorBuildError)
    }
}

impl<T> ToCryptoError<T> for Result<T, String> {
    fn aes_decrypt_error(self) -> Result<T, CryptoError> {
        self.map_err(|e| Whatever::without_source(e))
            .context(AesDecryptError)
    }

    fn aes_decryptor_build_error(self) -> Result<T, CryptoError> {
        self.map_err(|e| Whatever::without_source(e))
            .context(AesDecryptorBuildError)
    }
}
