//! Defines the functions necessary for decrypting AES-encrypted data returned by the interactsh servers.

/// Decrypt the provided data using the provided plain-text AES key
pub fn decrypt_data(aes_key: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, String> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "rustcrypto")] {
            rustcrypto_decrypt(aes_key, encrypted_data)
        } else if #[cfg(feature = "openssl")] {
            openssl_decrypt(aes_key, encrypted_data)
        } 
    }
}

/// Decrypt the provided data using the provided plain-text AES key (using RustCrypto libraries)
#[cfg(feature = "rustcrypto")]
fn rustcrypto_decrypt(aes_key: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, String> {
    use aes::cipher::{KeyIvInit, AsyncStreamCipher};
    type Aes128CfbDec = cfb_mode::Decryptor<aes::Aes128>;

    let iv = &encrypted_data[0..16];
    let decryptor = Aes128CfbDec::new(aes_key.into(), iv.into());
    let mut decrypted_data: Vec<u8> = Vec::with_capacity(encrypted_data.len());
    
    decryptor.decrypt_b2b(encrypted_data, &mut decrypted_data)
        .map_err(|e| format!("Error: {e}"))?;

    Ok(decrypted_data)
}


/// Decrypt the provided data using the provided plain-text AES key (using the OpenSSL library)
#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
fn openssl_decrypt(aes_key: &[u8], encrypted_data: &[u8]) -> Result<Vec<u8>, String> {
    let iv = &encrypted_data[0..16];
    let cipher = openssl::symm::Cipher::aes_128_cfb128();

    let decrypted_data = openssl::symm::decrypt(cipher, aes_key, Some(iv), encrypted_data)
        .map_err(|e| format!("Error: {e}"))?;
    
    Ok(decrypted_data)
}
