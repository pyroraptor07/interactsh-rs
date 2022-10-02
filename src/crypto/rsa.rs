//! Defines the wrapper structs and functions exposing the RSA key functionality needed
//! by the interactsh-rs client.

#[cfg(feature = "rustcrypto")]
use rsa::{RsaPrivateKey, RsaPublicKey};

#[cfg(feature = "openssl")]
use openssl::pkey::{Private, Public, PKey};

use crate::errors::{
    RsaDecryptError,
    RsaEncodePubKeyError,
    RsaGenError,
    RsaGetPubKeyError,
};


/// Wrapper struct for the RSA public key
pub struct RSAPubKey {
    #[cfg(feature = "rustcrypto")]
    rustcrypto_pubkey: RsaPublicKey,
    #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
    openssl_pubkey: PKey<Public>,
}

impl RSAPubKey {
    /// Encodes the public key as a base 64 encoded string
    pub fn b64_encode(&self) -> Result<String, RsaEncodePubKeyError> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                rustcrypto_fns::encode_public_key(&self.rustcrypto_pubkey)
            } else if #[cfg(feature = "openssl")] {
                openssl_fns::encode_public_key(&self.openssl_pubkey)
            } 
        }
    }
}

/// Wrapper struct for the RSA private key
#[derive(Clone)]
pub struct RSAPrivKey {
    #[cfg(feature = "rustcrypto")]
    rustcrypto_privkey: RsaPrivateKey,
    #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
    openssl_privkey: PKey<Private>,
}

impl std::fmt::Debug for RSAPrivKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<RSA Private Key>")
    }
}

impl RSAPrivKey {
    /// Generates a new RSA private key with the provided number of bits
    /// 
    /// Note: when using the "rustcrypto" feature in the debug build profile,
    /// this function can take some time (depending on the number of bits).
    pub fn generate(num_bits: usize) -> Result<Self, RsaGenError> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                rustcrypto_fns::get_rsa(num_bits)
            } else if #[cfg(feature = "openssl")] {
                openssl_fns::get_rsa(num_bits)
            } 
        }
    }

    /// Extracts the public key from the generated private key
    pub fn get_pub_key(&self) -> Result<RSAPubKey, RsaGetPubKeyError> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                rustcrypto_fns::get_public_key(&self.rustcrypto_privkey)
            } else if #[cfg(feature = "openssl")] {
                openssl_fns::get_public_key(&self.openssl_privkey)
            } 
        }
    }

    /// Decrypts the provided data using the provided SHA2 hash algorithm
    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, RsaDecryptError> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                rustcrypto_fns::decrypt_data(
                    &self.rustcrypto_privkey,
                    encrypted_data,
                )
            } else if #[cfg(feature = "openssl")] {
                openssl_fns::decrypt_data(
                    &self.openssl_privkey,
                    encrypted_data,
                )
            } 
        }
    }
}


#[cfg(feature = "rustcrypto")]
mod rustcrypto_fns {
    //! RustCrypto-specific RSA functions

    use digest::DynDigest;
    use rand::thread_rng;
    use rsa::{padding::PaddingScheme, pkcs8::{EncodePublicKey, LineEnding}};

    use super::*;

    /// Generates a new RSA private key with the provided number of bits
    pub(super) fn get_rsa(num_bits: usize) -> Result<RSAPrivKey, RsaGenError> {
        let rustcrypto_privkey = RsaPrivateKey::new(&mut thread_rng(), num_bits)?;
        let priv_key = RSAPrivKey {
            rustcrypto_privkey,
        };
    
        Ok(priv_key)
    }
    
    /// Decrypts the provided data using the provided SHA2 hash algorithm and RSA private key
    pub(super) fn decrypt_data(
        priv_key: &RsaPrivateKey,
        encrypted_data: &[u8]
    ) -> Result<Vec<u8>, RsaDecryptError> {
        let hasher: Box<dyn DynDigest> = Box::new(sha2::Sha256::default());
        let padding = PaddingScheme::OAEP { 
            digest: Box::clone(&hasher), 
            mgf_digest: hasher,
            label: None,
        };

        let decrypted_bytes = priv_key.decrypt(padding, encrypted_data)?;

        Ok(decrypted_bytes)
    }

    /// Extracts the public key from the provided private key
    pub(super) fn get_public_key(priv_key: &RsaPrivateKey) -> Result<RSAPubKey, RsaGetPubKeyError> {
        let pub_key = priv_key.to_public_key();

        Ok(RSAPubKey { rustcrypto_pubkey: pub_key })
    }

    /// Encodes the provided public key as a base 64 encoded string
    pub(super) fn encode_public_key(pub_key: &RsaPublicKey) -> Result<String, RsaEncodePubKeyError> {
        let pub_key_pem = pub_key.to_public_key_pem(LineEnding::LF)?;
        let pub_key_b64 = base64::encode(pub_key_pem);

        Ok(pub_key_b64)
    }
}

#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
mod openssl_fns {
    //! OpenSSL-specific RSA functions

    use openssl::md::Md;
    use openssl::rsa::Rsa;
    use openssl::pkey::PKeyRef;
    use openssl::pkey_ctx::PkeyCtx;
    use openssl::rsa::Padding;

    use crate::errors::RsaGenInnerError;
    use super::*;

    /// Generates a new RSA private key with the provided number of bits
    pub(super) fn get_rsa(num_bits: usize) -> Result<RSAPrivKey, RsaGenError> {
        let num_bits = if num_bits <= u32::MAX as usize {
            num_bits as u32
        } else {
            return Err(RsaGenInnerError::BitSize.into());
        };
    
        let rsa_key = Rsa::generate(num_bits)
            .map_err(|e| RsaGenInnerError::from(e))?;
        let openssl_privkey = PKey::from_rsa(rsa_key)
            .map_err(|e| RsaGenInnerError::from(e))?;
    
        let priv_key = RSAPrivKey {
            openssl_privkey,
        };
    
        Ok(priv_key)
    }
    
    /// Decrypts the provided data using the provided SHA2 hash algorithm and RSA private key
    pub(super) fn decrypt_data(
        priv_key: &PKeyRef<Private>,
        encrypted_data: &[u8],
    ) -> Result<Vec<u8>, RsaDecryptError> {
        let hasher = Md::sha256();
        let mut pkey_ctx = PkeyCtx::new(priv_key)?;
        pkey_ctx.decrypt_init()?;
        pkey_ctx.set_rsa_padding(Padding::PKCS1_OAEP)?;
        pkey_ctx.set_rsa_oaep_md(hasher)?;

        let mut decrypted_data = Vec::new();
        let _ = pkey_ctx.decrypt_to_vec(encrypted_data, &mut decrypted_data)?;

        Ok(decrypted_data)
    }

    /// Extracts the public key from the provided private key
    pub(super) fn get_public_key(priv_key: &PKeyRef<Private>) -> Result<RSAPubKey, RsaGetPubKeyError> {
        let pub_key_pem = priv_key.public_key_to_pem()?;
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem)?;
        let pkey_pub_key = PKey::from_rsa(pub_key)?;

        Ok(RSAPubKey { openssl_pubkey: pkey_pub_key })
    }

    /// Encodes the provided public key as a base 64 encoded string
    pub(super) fn encode_public_key(pub_key: &PKeyRef<Public>) -> Result<String, RsaEncodePubKeyError> {
        let pub_key_pem = pub_key.public_key_to_pem()?;
        let pub_key_b64 = base64::encode(pub_key_pem);

        Ok(pub_key_b64)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsa_private_key_generates_successfully_with_2048_bits() {
        let _rsa_private_key = RSAPrivKey::generate(2048).expect("RSA private key generation failed with 2048 bits");
    }

    #[test]
    fn rsa_public_key_extraction_works_successfully() {
        let rsa_private_key = RSAPrivKey::generate(2048).expect("RSA private key generation failed with 2048 bits");

        let _rsa_public_key = rsa_private_key.get_pub_key().expect("Failed to extract the public key");
    }

    #[test]
    fn rsa_public_key_b64_encode_works_successfully() {
        let rsa_private_key = RSAPrivKey::generate(2048).expect("RSA private key generation failed with 2048 bits");

        let rsa_public_key = rsa_private_key.get_pub_key().expect("Failed to extract the public key");

        let _encoded_public_key = rsa_public_key.b64_encode().expect("Failed to encode the public key");
    }
}
