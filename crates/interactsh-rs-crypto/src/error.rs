use snafu::prelude::*;

/// The main error type that should be returned by any implementer of the
/// [AesDecryptor](crate::aes::AesDecryptor) and XXX traits
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum CryptoError {
    #[snafu(display("Aes decryption error: {source}"))]
    AesDecryptError { source: Box<dyn std::error::Error> },

    #[snafu(display("Aes decryptor build error: {source}"))]
    AesDecryptorBuildError { source: Box<dyn std::error::Error> },
}

impl CryptoError {
    pub fn new_aes_decrypt_error(source: impl Into<Box<dyn std::error::Error>>) -> CryptoError {
        Self::AesDecryptError {
            source: Into::into(source),
        }
    }

    pub fn new_aes_decryptor_build_error(
        source: impl Into<Box<dyn std::error::Error>>,
    ) -> CryptoError {
        Self::AesDecryptorBuildError {
            source: Into::into(source),
        }
    }
}
