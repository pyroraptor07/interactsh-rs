use base64::engine::general_purpose;
use base64::Engine as _;
use rsa::pkcs8::{EncodePublicKey, LineEnding};
use rsa::Oaep;

use crate::error::IntoCryptoError;
use crate::pkey::PrivateKey;

#[derive(Debug)]
pub struct RustCryptoPrivateKeySize(usize);

impl Default for RustCryptoPrivateKeySize {
    fn default() -> Self {
        Self(2048)
    }
}

impl From<usize> for RustCryptoPrivateKeySize {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

pub struct RustCryptoPrivateKey {
    priv_key: rsa::RsaPrivateKey,
}

impl PrivateKey for RustCryptoPrivateKey {
    type Settings = RustCryptoPrivateKeySize;

    fn new_with_settings(settings: Self::Settings) -> Result<Self, crate::error::CryptoError>
    where
        Self: Sized,
    {
        let priv_key =
            rsa::RsaPrivateKey::new(&mut rand::thread_rng(), settings.0).map_err(|e| {
                e.to_pkey_gen_error(Some(format!(
                    "Private key generation failed, provided bit value was {}",
                    settings.0
                )))
            })?;

        Ok(Self { priv_key })
    }

    fn get_encoded_pub_key(&self) -> Result<String, crate::error::CryptoError> {
        let pub_key = self.priv_key.to_public_key();
        let pub_key_pem = pub_key.to_public_key_pem(LineEnding::LF).map_err(|e| {
            e.to_pkey_pub_encode_error(Some("Failed to get PEM-formatted public key"))
        })?;
        let pub_key_b64 = general_purpose::STANDARD.encode(pub_key_pem);

        Ok(pub_key_b64)
    }

    fn decrypt_data(
        &self,
        encrypted_data: bytes::Bytes,
    ) -> Result<bytes::Bytes, crate::error::CryptoError> {
        let padding = Oaep::new::<sha2::Sha256>();
        let decrypted_bytes = self
            .priv_key
            .decrypt(padding, &encrypted_data)
            .map_err(|e| e.to_pkey_decrypt_error(Some("RSA decryption failed")))?;

        Ok(decrypted_bytes.into())
    }

    fn secure_drop(&mut self) {
        // RSAPrivateKey implements ZeroizeOnDrop, so nothing additional to do here
    }
}
