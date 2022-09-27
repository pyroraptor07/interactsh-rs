//! # Interactsh-rs
//! A Rust client library for working with Interact.sh servers.


#![allow(unused)] // REMOVE BEFORE PUBLISHING
#![cfg_attr(feature = "nightly", feature(doc_auto_cfg))]
#![cfg_attr(feature = "nightly", feature(error_generic_member_access))]
#![cfg_attr(feature = "nightly", feature(provide_any))]

// Compile time feature checks
#[cfg(not(any(feature = "reqwest-rustls-tls", feature = "reqwest-native-tls")))]
compile_error!("One of the following features MUST be enabled:\n- \"reqwest-rustls-tls\"\n- \"reqwest-native-tls\"\n");

#[cfg(not(any(feature = "rustcrypto", feature = "openssl")))]
compile_error!("One of the following features MUST be enabled:\n- \"rustcrypto\"\n- \"openssl\"\n");

#[cfg(any(feature = "rustcrypto", feature = "openssl"))]
pub mod crypto;

#[cfg(any(feature = "reqwest-rustls-tls", feature = "reqwest-native-tls"))]
pub mod client;

pub mod errors;
