use bytes::Bytes;

use crate::error::CryptoError;


pub trait PrivateKey {
    type Settings: Default;

    fn new_with_settings(settings: Self::Settings) -> Result<Self, CryptoError>
    where
        Self: Sized;

    fn get_encoded_pub_key(&self) -> Result<String, CryptoError>;

    fn decrypt_data(&self, encrypted_data: Bytes) -> Result<Bytes, CryptoError>;

    fn secure_drop(&mut self);
}
