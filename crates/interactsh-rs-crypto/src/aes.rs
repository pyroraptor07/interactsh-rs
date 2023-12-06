use bytes::Bytes;

use crate::error::CryptoError;

/// Defines the functions used by the Interactsh-rs client to decrypt AES-encrypted data
pub trait AesDecryptor {
    /// The settings type used to create the AES decryptor
    ///
    /// Ideally, the default settings should correspond to the AES settings used by the
    /// public Interactsh servers.
    type Settings: Default;

    /// Build the AES decryptor with the provided settings
    ///
    /// When implementing this, you should convert any errors to [CryptoError::AesDecryptorBuildError]
    /// type.
    fn new_with_settings(settings: Self::Settings) -> Result<Self, CryptoError>
    where
        Self: Sized;

    /// The primary AES decryption function
    ///
    /// When implementing this, you should convert any errors to [CryptoError::AesDecryptError]
    /// type.
    fn decrypt_data(&self, aes_key: Bytes, encrypted_data: Bytes) -> Result<Bytes, CryptoError>;
}
