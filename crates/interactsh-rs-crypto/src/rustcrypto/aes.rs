use aes::cipher::{AsyncStreamCipher, KeyIvInit};
use bytes::{Bytes, BytesMut};

use crate::aes::AesDecryptor;
use crate::error::CryptoError;

const IV_LEN: usize = 16;

#[derive(Debug)]
pub enum AesKeySize {
    Aes128,
    Aes192,
    Aes256,
}

#[derive(Debug)]
pub struct AesSettings {
    pub key_size: AesKeySize,
}

impl Default for AesSettings {
    fn default() -> Self {
        Self {
            key_size: AesKeySize::Aes256,
        }
    }
}

#[derive(Debug)]
pub struct RustCryptoAesCfbDecryptor {
    settings: AesSettings,
}

impl AesDecryptor for RustCryptoAesCfbDecryptor {
    type Settings = AesSettings;

    fn new_with_settings(settings: Self::Settings) -> Result<Self, CryptoError>
    where
        Self: Sized,
    {
        let me = Self { settings };
        Ok(me)
    }

    fn decrypt_data(&self, aes_key: Bytes, encrypted_data: Bytes) -> Result<Bytes, CryptoError> {
        let iv = &encrypted_data[0..IV_LEN];
        let mut buffer = BytesMut::from(Bytes::copy_from_slice(&encrypted_data[IV_LEN..]).as_ref());

        match &self.settings.key_size {
            AesKeySize::Aes128 => {
                let decryptor =
                    cfb_mode::Decryptor::<aes::Aes128>::new(aes_key.as_ref().into(), iv.into());
                decryptor.decrypt(buffer.as_mut());
            }
            AesKeySize::Aes192 => {
                let decryptor =
                    cfb_mode::Decryptor::<aes::Aes192>::new(aes_key.as_ref().into(), iv.into());
                decryptor.decrypt(buffer.as_mut());
            }
            AesKeySize::Aes256 => {
                let decryptor =
                    cfb_mode::Decryptor::<aes::Aes256>::new(aes_key.as_ref().into(), iv.into());
                decryptor.decrypt(buffer.as_mut());
            }
        }

        Ok(buffer.into())
    }

    fn secure_drop(&mut self) {
        // nothing to handle securely before drop
    }
}
