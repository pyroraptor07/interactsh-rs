//! The errors used within this crate


#[cfg(feature = "rustcrypto")]
mod rustcrypto_errors;

#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
mod openssl_errors;

#[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
mod client_errors;


#[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
pub use client_errors::*;
#[cfg(all(feature = "openssl", not(feature = "rustcrypto")))]
pub(crate) use openssl_errors::*;
#[cfg(feature = "rustcrypto")]
pub(crate) use rustcrypto_errors::*;
