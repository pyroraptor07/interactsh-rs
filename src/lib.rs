#[cfg(not(any(feature = "reqwest-rustls", feature = "reqwest-native-tls")))]
compile_error!("Either feature \"reqwest-rustls\" or feature \"reqwest-native-tls\" must be enabled!");

#[cfg(not(any(feature = "rsa-crate", feature = "openssl")))]
compile_error!("Either feature \"rsa-crate\" or feature \"openssl\" must be enabled!");

mod rsa;

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
