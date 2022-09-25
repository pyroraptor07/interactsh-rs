#[cfg(feature = "rustcrypto")]
mod rustcrypto_errors;

#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
mod openssl_errors;

mod client_errors;


#[cfg(feature = "rustcrypto")]
pub use rustcrypto_errors::*;

#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
pub use openssl_errors::*;

pub use client_errors::*;

