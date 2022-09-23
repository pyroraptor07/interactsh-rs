//! Defines the wrapper structs and functions exposing the RSA key functionality needed
//! by the interactsh-rs client.

#[cfg(feature = "rustcrypto")]
use rsa::{RsaPrivateKey, RsaPublicKey};

#[cfg(feature = "openssl")]
use openssl::pkey::{Private, Public, PKey};

use super::hash::Sha2HashAlgo;


/// Wrapper struct for the RSA public key
pub struct RSAPubKey {
    #[cfg(feature = "rustcrypto")]
    rustcrypto_pubkey: RsaPublicKey,
    #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
    openssl_pubkey: PKey<Public>,
}

impl RSAPubKey {
    /// Encodes the public key as a base 64 encoded string
    pub fn b64_encode(&self) -> Result<String, String> {
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
pub struct RSAPrivKey {
    #[cfg(feature = "rustcrypto")]
    rustcrypto_privkey: RsaPrivateKey,
    #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
    openssl_privkey: PKey<Private>,
}

impl RSAPrivKey {
    /// Generates a new RSA private key with the provided number of bits
    /// 
    /// Note: when using the "rustcrypto" feature in the debug build profile,
    /// this function can take some time (depending on the number of bits).
    pub fn generate(num_bits: usize) -> Result<Self, String> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                rustcrypto_fns::get_rsa(num_bits)
            } else if #[cfg(feature = "openssl")] {
                openssl_fns::get_rsa(num_bits)
            } 
        }
    }

    /// Extracts the public key from the generated private key
    pub fn get_pub_key(&self) -> Result<RSAPubKey, String> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                rustcrypto_fns::get_public_key(&self.rustcrypto_privkey)
            } else if #[cfg(feature = "openssl")] {
                openssl_fns::get_public_key(&self.openssl_privkey)
            } 
        }
    }

    /// Decrypts the provided data using the provided SHA2 hash algorithm
    pub fn decrypt_data(&self, hash_algorithm: &Sha2HashAlgo, encrypted_data: &[u8]) -> Result<Vec<u8>, String> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                rustcrypto_fns::decrypt_data(
                    &self.rustcrypto_privkey,
                    hash_algorithm.get_rustcrypto_hash(),
                    encrypted_data,
                )
            } else if #[cfg(feature = "openssl")] {
                openssl_fns::decrypt_data(
                    &self.openssl_privkey,
                    hash_algorithm.get_openssl_hash(),
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
    pub(super) fn get_rsa(num_bits: usize) -> Result<RSAPrivKey, String> {
        let rustcrypto_privkey = RsaPrivateKey::new(&mut thread_rng(), num_bits)
            .map_err(|e| format!("Error: {}", e))?;
        
        let priv_key = RSAPrivKey {
            rustcrypto_privkey,
        };
    
        Ok(priv_key)
    }
    
    /// Decrypts the provided data using the provided SHA2 hash algorithm and RSA private key
    pub(super) fn decrypt_data(
        priv_key: &RsaPrivateKey,
        hasher: Box::<dyn DynDigest>,
        encrypted_data: &[u8]
    ) -> Result<Vec<u8>, String> {
        let padding = PaddingScheme::OAEP { 
            digest: Box::clone(&hasher), 
            mgf_digest: hasher,
            label: None,
        };

        let decrypted_bytes = priv_key.decrypt(padding, encrypted_data)
            .map_err(|e| format!("Error: {}", e))?;

        Ok(decrypted_bytes)
    }

    /// Extracts the public key from the provided private key
    pub(super) fn get_public_key(priv_key: &RsaPrivateKey) -> Result<RSAPubKey, String> {
        let pub_key = priv_key.to_public_key();

        Ok(RSAPubKey { rustcrypto_pubkey: pub_key })
    }

    /// Encodes the provided public key as a base 64 encoded string
    pub(super) fn encode_public_key(pub_key: &RsaPublicKey) -> Result<String, String> {
        let pub_key_pem = pub_key.to_public_key_pem(LineEnding::LF)
            .map_err(|e| format!("Error: {}", e))?;

        let pub_key_b64 = base64::encode(pub_key_pem);

        Ok(pub_key_b64)
    }
}

#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
mod openssl_fns {
    //! OpenSSL-specific RSA functions

    use std::fmt::format;

    use openssl::md::MdRef;
    use openssl::rsa::Rsa;
    use openssl::pkey::PKeyRef;
    use openssl::pkey_ctx::PkeyCtx;
    use openssl::rsa::Padding;

    use super::*;

    /// Generates a new RSA private key with the provided number of bits
    pub(super) fn get_rsa(num_bits: usize) -> Result<RSAPrivKey, String> {
        let num_bits = if num_bits <= u32::MAX as usize {
            num_bits as u32
        } else {
            return Err(String::from("Error: Requested bits size is too large"));
        };
    
    
        let rsa_key = Rsa::generate(num_bits).map_err(|e| format!("Error: {}", e))?;
        let openssl_privkey = PKey::from_rsa(rsa_key)
            .map_err(|e| format!("Error: {}", e))?;
    
        let priv_key = RSAPrivKey {
            openssl_privkey,
        };
    
        Ok(priv_key)
    }
    
    /// Decrypts the provided data using the provided SHA2 hash algorithm and RSA private key
    pub(super) fn decrypt_data(
        priv_key: &PKeyRef<Private>,
        hasher: &MdRef,
        encrypted_data: &[u8],
    ) -> Result<Vec<u8>, String> {
        let mut pkey_ctx = PkeyCtx::new(priv_key)
            .map_err(|e| format!("Error: {}", e))?;

        pkey_ctx.set_rsa_padding(Padding::PKCS1_OAEP)
            .map_err(|e| format!("Error: {}", e))?;

        pkey_ctx.set_rsa_oaep_md(hasher)
            .map_err(|e| format!("Error: {}", e))?;

        let mut decrypted_data = Vec::new();
        let _ = pkey_ctx.decrypt_to_vec(encrypted_data, &mut decrypted_data)
            .map_err(|e| format!("Error: {}", e))?;

        Ok(decrypted_data)
    }

    /// Extracts the public key from the provided private key
    pub(super) fn get_public_key(priv_key: &PKeyRef<Private>) -> Result<RSAPubKey, String> {
        let pub_key_pem = priv_key.public_key_to_pem()
            .map_err(|e| format!("Error: {}", e))?;

        let pub_key = Rsa::public_key_from_pem(&pub_key_pem)
            .map_err(|e| format!("Error: {}", e))?;

        let pkey_pub_key = PKey::from_rsa(pub_key)
            .map_err(|e| format!("Error: {}", e))?;

        Ok(RSAPubKey { openssl_pubkey: pkey_pub_key })
    }

    /// Encodes the provided public key as a base 64 encoded string
    pub(super) fn encode_public_key(pub_key: &PKeyRef<Public>) -> Result<String, String> {
        let pub_key_pem = pub_key.public_key_to_pem()
            .map_err(|e| format!("Error: {}", e))?;

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
