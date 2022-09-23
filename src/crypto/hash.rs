//! Defines the SHA2 hash algorithm wrapper types used by the RSA keys.

#[cfg(feature = "rustcrypto")]
use digest::DynDigest;

#[cfg(feature = "openssl")]
use openssl::md::{Md, MdRef};

/// Enum for the SHA2 hash algorithm types that are supported
pub enum Sha2HashAlgoType {
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}


/// Wrapper struct around the SHA2 hash algorithm types used by the RustCrypto and OpenSSL crates
pub struct Sha2HashAlgo {
    #[cfg(feature = "rustcrypto")]
    rustcrypto_hash: Box<dyn DynDigest>,

    #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
    openssl_hash: &'static MdRef,
}

impl Sha2HashAlgo {
    /// Create a new Sha2HashAlgo struct for the given SHA2 has algorithm
    pub fn new(algo_type: Sha2HashAlgoType) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rustcrypto")] {
                let rustcrypto_hash = rustcrypto_get_sha2(algo_type);
            } else if #[cfg(feature = "openssl")] {
                let openssl_hash = openssl_get_sha2(algo_type);
            } 
        }

        Self {
            #[cfg(feature = "rustcrypto")]
            rustcrypto_hash,

            #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
            openssl_hash,
        }
    }

    /// Return the associated RustCrypto SHA2 hash algorithm type
    #[cfg(feature = "rustcrypto")]
    pub fn get_rustcrypto_hash(&self) -> Box<dyn DynDigest> {
        Box::clone(&self.rustcrypto_hash)
    }

    /// Return the associated OpenSSL SHA2 hash algorithm type
    #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
    pub fn get_openssl_hash(&self) -> &'static MdRef {
        self.openssl_hash
    }
}

/// Return the associated RustCrypto SHA2 hash algorithm type
#[cfg(feature = "rustcrypto")]
fn rustcrypto_get_sha2(algo_type: Sha2HashAlgoType) -> Box<dyn DynDigest> {
    match algo_type {
        Sha2HashAlgoType::Sha224 => Box::new(sha2::Sha224::default()),
        Sha2HashAlgoType::Sha256 => Box::new(sha2::Sha256::default()),
        Sha2HashAlgoType::Sha384 => Box::new(sha2::Sha384::default()),
        Sha2HashAlgoType::Sha512 => Box::new(sha2::Sha512::default()),
    }
}

/// Return the associated OpenSSL SHA2 hash algorithm type
#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
fn openssl_get_sha2(algo_type: Sha2HashAlgoType) -> &'static MdRef {
    match algo_type {
        Sha2HashAlgoType::Sha224 => Md::sha224(),
        Sha2HashAlgoType::Sha256 => Md::sha256(),
        Sha2HashAlgoType::Sha384 => Md::sha384(),
        Sha2HashAlgoType::Sha512 => Md::sha512(),
    }
}