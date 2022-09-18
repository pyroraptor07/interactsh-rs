#![allow(unused)] // REMOVE BEFORE PUBLISHING

// Compile time feature checks
#[cfg(not(any(feature = "reqwest-rustls-tls", feature = "reqwest-native-tls")))]
compile_error!("One of the following features MUST be enabled:\n- \"reqwest-rustls-tls\"\n- \"reqwest-native-tls\"\n");

#[cfg(not(any(feature = "rustcrypto", feature = "openssl")))]
compile_error!("One of the following features MUST be enabled:\n- \"rustcrypto\"\n- \"openssl\"\n");


mod crypto;
mod client;

// Leaving this in as an example for now, remove later
//
// fn test() {
//     cfg_if::cfg_if! {
//         if #[cfg(all(feature = "rsa-crate", feature = "openssl"))] {
//             println!("Both 'rsa-crate' and 'openssl' are enabled!");
//         } else if #[cfg(feature = "openssl")] {
//             println!("Only 'openssl' is enabled!");
//         } else if #[cfg(feature = "rsa-crate")] {
//             println!("Only 'rsa-crate' is enabled!");
//         }
//     }
// }
