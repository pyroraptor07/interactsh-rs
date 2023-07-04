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
    /// types (use [ToCryptoError::aes_decryptor_build_error](crate::error::ToCryptoError::aes_decryptor_build_error)
    /// to streamline this).
    fn new_with_settings(settings: Self::Settings) -> Result<Self, CryptoError>;

    /// The primary AES decryption function
    ///
    /// When implementing this, you should convert any errors to [CryptoError::AesDecryptError]
    /// types (use [ToCryptoError::aes_decrypt_error](crate::error::ToCryptoError::aes_decrypt_error)
    /// to streamline this).
    fn decrypt_data(&self, aes_key: Bytes, encrypted_data: Bytes) -> Result<Bytes, CryptoError>;
}
