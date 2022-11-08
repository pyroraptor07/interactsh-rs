pub(crate) use errors_to_reexport::crypto_error;
pub use errors_to_reexport::CryptoError;


mod errors_to_reexport {
    use snafu::prelude::*;
    use snafu::Backtrace;


    cfg_if::cfg_if! {
        if #[cfg(feature = "rustcrypto")] {
            pub use RustCryptoError as CryptoError;
        } else if #[cfg(feature = "openssl")] {
            pub use OpensslError as CryptoError;
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "rustcrypto")] {
            pub use rustcrypto_error as crypto_error;
        } else if #[cfg(feature = "openssl")] {
            pub use openssl_error as crypto_error;
        }
    }


    /// Errors returned for cryptography operations
    #[cfg(feature = "rustcrypto")]
    #[derive(Debug, Snafu)]
    #[snafu(module(rustcrypto_error), context(suffix(false)), visibility(pub))]
    pub enum RustCryptoError {
        #[snafu(display("Failed to decode the data using base 64 encoding"))]
        Base64DecodeAes {
            source: base64::DecodeError,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to encode the RSA public key as a base 64 string"))]
        Base64EncodeRsaPub {
            source: rsa::pkcs8::spki::Error,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to generate the RSA private key"))]
        RsaGen {
            source: rsa::errors::Error,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to decrypt the data with the provided RSA private key"))]
        RsaDecrypt {
            source: rsa::errors::Error,
            backtrace: Backtrace,
        },
    }


    /// Errors returned for cryptography operations
    #[cfg(feature = "openssl")]
    #[derive(Debug, Snafu)]
    #[snafu(module, context(suffix(false)), visibility(pub))]
    pub enum OpensslError {
        #[snafu(display("Unable to decrypt data with provided AES key"))]
        AesDecrypt {
            source: openssl::error::ErrorStack,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to decode the data using base 64 encoding"))]
        Base64DecodeAes {
            source: base64::DecodeError,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to encode the RSA public key as a base 64 string"))]
        Base64EncodeRsaPub {
            source: openssl::error::ErrorStack,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to generate the RSA private key"))]
        RsaGen {
            source: openssl::error::ErrorStack,
            backtrace: Backtrace,
        },

        #[snafu(display("Requested RSA key bit size is too large (bitsize: {bitsize}"))]
        RsaBitSize {
            bitsize: usize,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to decrypt the data with the provided RSA private key"))]
        RsaDecrypt {
            source: openssl::error::ErrorStack,
            backtrace: Backtrace,
        },

        #[snafu(display("Failed to extract the RSA public key from the RSA private key"))]
        RsaGetPubKey {
            source: openssl::error::ErrorStack,
            backtrace: Backtrace,
        },
    }
}
