use thiserror::Error;


#[derive(Error, Debug)]
#[error("Unable to decrypt data with provided AES key")]
pub struct AesDecryptError {
    #[from]
    source: openssl::error::ErrorStack,
}


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


#[derive(Error, Debug)]
#[error("Failed to generate the RSA private key")]
pub struct RsaGenError {
    #[from]
    source: RsaGenInnerError,
}


#[derive(Error, Debug)]
#[error("Failed to decrypt the data with the provided RSA private key")]
pub struct RsaDecryptError {
    #[from]
    source: openssl::error::ErrorStack,
}


#[derive(Error, Debug)]
#[error("Failed to extract the RSA public key from the RSA private key")]
pub struct RsaGetPubKeyError {
    #[from]
    source: openssl::error::ErrorStack
}


#[derive(Error, Debug)]
#[error("Failed to encode the RSA public key as a base 64 string")]
pub struct RsaEncodePubKeyError {
    #[from]
    source: openssl::error::ErrorStack
}