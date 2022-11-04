cfg_if::cfg_if! {
    if #[cfg(feature = "rustcrypto")] {
        pub use rustcrypto_errors::*;
    } else if #[cfg(feature = "openssl")] {
        pub use openssl_errors::*;
    }
}


#[cfg(feature = "rustcrypto")]
pub mod rustcrypto_errors {
    use snafu::prelude::*;
    use snafu::Backtrace;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub enum CryptoError {
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
}


#[cfg(feature = "openssl")]
pub mod openssl_errors {
    use snafu::prelude::*;
    use snafu::Backtrace;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub enum CryptoError {
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
