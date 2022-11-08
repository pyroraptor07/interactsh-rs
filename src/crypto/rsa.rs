//! Defines the wrapper structs and functions exposing the RSA key functionality needed
//! by the interactsh-rs client.
#[cfg(feature = "openssl")]
use openssl::pkey::{PKey, Private, Public};
#[cfg(feature = "rustcrypto")]
use rsa::{RsaPrivateKey, RsaPublicKey};

use super::errors::{crypto_error, CryptoError};


/// Wrapper struct for the RSA public key
pub struct RSAPubKey {
    #[cfg(feature = "rustcrypto")]
    rustcrypto_pubkey: RsaPublicKey,
    #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
    openssl_pubkey: PKey<Public>,
}

impl RSAPubKey {
    /// Encodes the public key as a base 64 encoded string
    pub(crate) fn b64_encode(&self) -> Result<String, CryptoError> {
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
    pub(crate) fn generate(num_bits: usize) -> Result<Self, CryptoError> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                rustcrypto_fns::get_rsa(num_bits)
            } else if #[cfg(feature = "openssl")] {
                openssl_fns::get_rsa(num_bits)
            }
        }
    }

    /// Extracts the public key from the generated private key
    pub(crate) fn get_pub_key(&self) -> Result<RSAPubKey, CryptoError> {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                rustcrypto_fns::get_public_key(&self.rustcrypto_privkey)
            } else if #[cfg(feature = "openssl")] {
                openssl_fns::get_public_key(&self.openssl_privkey)
            }
        }
    }

    /// Decrypts the provided data using the provided SHA2 hash algorithm
    pub(crate) fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, CryptoError> {
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
    use rsa::padding::PaddingScheme;
    use rsa::pkcs8::{EncodePublicKey, LineEnding};
    use snafu::ResultExt;

    use super::*;

    /// Generates a new RSA private key with the provided number of bits
    pub(super) fn get_rsa(num_bits: usize) -> Result<RSAPrivKey, CryptoError> {
        let rustcrypto_privkey =
            RsaPrivateKey::new(&mut thread_rng(), num_bits).context(crypto_error::RsaGen)?;
        let priv_key = RSAPrivKey { rustcrypto_privkey };

        Ok(priv_key)
    }

    /// Decrypts the provided data using the provided SHA2 hash algorithm and RSA private key
    pub(super) fn decrypt_data(
        priv_key: &RsaPrivateKey,
        encrypted_data: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let hasher: Box<dyn DynDigest> = Box::new(sha2::Sha256::default());
        let padding = PaddingScheme::OAEP {
            digest: Box::clone(&hasher),
            mgf_digest: hasher,
            label: None,
        };

        let decrypted_bytes = priv_key
            .decrypt(padding, encrypted_data)
            .context(crypto_error::RsaDecrypt)?;

        Ok(decrypted_bytes)
    }

    /// Extracts the public key from the provided private key
    pub(super) fn get_public_key(priv_key: &RsaPrivateKey) -> Result<RSAPubKey, CryptoError> {
        let pub_key = priv_key.to_public_key();

        Ok(RSAPubKey {
            rustcrypto_pubkey: pub_key,
        })
    }

    /// Encodes the provided public key as a base 64 encoded string
    pub(super) fn encode_public_key(pub_key: &RsaPublicKey) -> Result<String, CryptoError> {
        let pub_key_pem = pub_key
            .to_public_key_pem(LineEnding::LF)
            .context(crypto_error::Base64EncodeRsaPub)?;
        let pub_key_b64 = base64::encode(pub_key_pem);

        Ok(pub_key_b64)
    }
}

#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
mod openssl_fns {
    //! OpenSSL-specific RSA functions

    use openssl::md::Md;
    use openssl::pkey::PKeyRef;
    use openssl::pkey_ctx::PkeyCtx;
    use openssl::rsa::{Padding, Rsa};
    use snafu::{ensure, ResultExt};

    use super::*;

    /// Generates a new RSA private key with the provided number of bits
    pub(super) fn get_rsa(num_bits: usize) -> Result<RSAPrivKey, CryptoError> {
        ensure!(
            num_bits <= u32::MAX as usize,
            crypto_error::RsaBitSize { bitsize: num_bits }
        );

        let num_bits = num_bits as u32;

        let rsa_key = Rsa::generate(num_bits).context(crypto_error::RsaGen)?;
        let openssl_privkey = PKey::from_rsa(rsa_key).context(crypto_error::RsaGen)?;

        let priv_key = RSAPrivKey { openssl_privkey };

        Ok(priv_key)
    }

    /// Decrypts the provided data using the provided SHA2 hash algorithm and RSA private key
    pub(super) fn decrypt_data(
        priv_key: &PKeyRef<Private>,
        encrypted_data: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let hasher = Md::sha256();
        let mut pkey_ctx = PkeyCtx::new(priv_key).context(crypto_error::RsaDecrypt)?;
        pkey_ctx.decrypt_init().context(crypto_error::RsaDecrypt)?;
        pkey_ctx
            .set_rsa_padding(Padding::PKCS1_OAEP)
            .context(crypto_error::RsaDecrypt)?;
        pkey_ctx
            .set_rsa_oaep_md(hasher)
            .context(crypto_error::RsaDecrypt)?;

        let mut decrypted_data = Vec::new();
        let _ = pkey_ctx
            .decrypt_to_vec(encrypted_data, &mut decrypted_data)
            .context(crypto_error::RsaDecrypt)?;

        Ok(decrypted_data)
    }

    /// Extracts the public key from the provided private key
    pub(super) fn get_public_key(priv_key: &PKeyRef<Private>) -> Result<RSAPubKey, CryptoError> {
        let pub_key_pem = priv_key
            .public_key_to_pem()
            .context(crypto_error::RsaGetPubKey)?;
        let pub_key = Rsa::public_key_from_pem(&pub_key_pem).context(crypto_error::RsaGetPubKey)?;
        let pkey_pub_key = PKey::from_rsa(pub_key).context(crypto_error::RsaGetPubKey)?;

        Ok(RSAPubKey {
            openssl_pubkey: pkey_pub_key,
        })
    }

    /// Encodes the provided public key as a base 64 encoded string
    pub(super) fn encode_public_key(pub_key: &PKeyRef<Public>) -> Result<String, CryptoError> {
        let pub_key_pem = pub_key
            .public_key_to_pem()
            .context(crypto_error::Base64EncodeRsaPub)?;
        let pub_key_b64 = base64::encode(pub_key_pem);

        Ok(pub_key_b64)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsa_private_key_generates_successfully_with_2048_bits() {
        let _rsa_private_key =
            RSAPrivKey::generate(2048).expect("RSA private key generation failed with 2048 bits");
    }

    #[test]
    fn rsa_public_key_extraction_works_successfully() {
        let rsa_private_key =
            RSAPrivKey::generate(2048).expect("RSA private key generation failed with 2048 bits");

        let _rsa_public_key = rsa_private_key
            .get_pub_key()
            .expect("Failed to extract the public key");
    }

    #[test]
    fn rsa_public_key_b64_encode_works_successfully() {
        let rsa_private_key =
            RSAPrivKey::generate(2048).expect("RSA private key generation failed with 2048 bits");

        let rsa_public_key = rsa_private_key
            .get_pub_key()
            .expect("Failed to extract the public key");

        let _encoded_public_key = rsa_public_key
            .b64_encode()
            .expect("Failed to encode the public key");
    }
}
