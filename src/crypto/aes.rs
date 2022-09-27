//! Defines the functions necessary for decrypting AES-encrypted data returned by the Interactsh servers.

use crate::errors::AesDecryptError;

/// Decrypt the provided data using the provided plain-text AES key
pub fn decrypt_data(aes_key: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, AesDecryptError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "rustcrypto")] {
            rustcrypto_decrypt(aes_key, &encrypted_data)
        } else if #[cfg(feature = "openssl")] {
            openssl_decrypt(aes_key, &encrypted_data)
        } 
    }
}

/// Decrypt the provided data using the provided plain-text AES key (using RustCrypto libraries)
#[cfg(feature = "rustcrypto")]
fn rustcrypto_decrypt(aes_key: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, AesDecryptError> {
    use aes::cipher::{KeyIvInit, AsyncStreamCipher};
    type Aes256CfbDec = cfb_mode::Decryptor<aes::Aes256>;

    let iv = &encrypted_data[0..16];

    let decryptor = Aes256CfbDec::new(aes_key.into(), iv.into());

    let mut decrypted_data = encrypted_data[16..].to_vec();
    decryptor.decrypt(&mut decrypted_data);

    Ok(decrypted_data)
}


/// Decrypt the provided data using the provided plain-text AES key (using the OpenSSL library)
#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
fn openssl_decrypt(aes_key: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, AesDecryptError> {
    let iv = &encrypted_data[0..16];
    let cipher = openssl::symm::Cipher::aes_256_cfb128();
    let sliced_encrypted_data = &encrypted_data[16..];

    let decrypted_data = openssl::symm::decrypt(cipher, aes_key, Some(iv), sliced_encrypted_data)?;
    
    Ok(decrypted_data)
}
