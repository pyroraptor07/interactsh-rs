#[cfg(feature = "rustcrypto")]
use digest::DynDigest;

#[cfg(feature = "openssl")]
use openssl::md::{Md, MdRef};

pub enum Sha2HashAlgoType {
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}


pub struct Sha2HashAlgo {
    #[cfg(feature = "rustcrypto")]
    rustcrypto_hash: Box<dyn DynDigest>,

    #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
    openssl_hash: &'static MdRef,
}

impl Sha2HashAlgo {    
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

    #[cfg(feature = "rustcrypto")]
    pub fn get_rustcrypto_hash(&self) -> Box<dyn DynDigest> {
        Box::clone(&self.rustcrypto_hash)
    }

    #[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
    pub fn get_openssl_hash(&self) -> &'static MdRef {
        self.openssl_hash
    }
}

#[cfg(feature = "rustcrypto")]
fn rustcrypto_get_sha2(algo_type: Sha2HashAlgoType) -> Box<dyn DynDigest> {
    match algo_type {
        Sha2HashAlgoType::Sha224 => Box::new(sha2::Sha224::default()),
        Sha2HashAlgoType::Sha256 => Box::new(sha2::Sha256::default()),
        Sha2HashAlgoType::Sha384 => Box::new(sha2::Sha384::default()),
        Sha2HashAlgoType::Sha512 => Box::new(sha2::Sha512::default()),
    }
}

#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
fn openssl_get_sha2(algo_type: Sha2HashAlgoType) -> &'static MdRef {
    match algo_type {
        Sha2HashAlgoType::Sha224 => Md::sha224(),
        Sha2HashAlgoType::Sha256 => Md::sha256(),
        Sha2HashAlgoType::Sha384 => Md::sha384(),
        Sha2HashAlgoType::Sha512 => Md::sha512(),
    }
}