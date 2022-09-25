use thiserror::Error;


#[derive(Error, Debug)]
#[error("Unable to decrypt data with provided AES key")]
pub struct AesDecryptError {
    #[from]
    source: inout::NotEqualError,
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
    source: rsa::pkcs8::spki::Error
}
