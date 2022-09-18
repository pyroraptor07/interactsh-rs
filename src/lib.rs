// Compile time feature checks
#[cfg(not(any(feature = "reqwest-rustls-tls", feature = "reqwest-native-tls")))]
compile_error!("One of the following features MUST be enabled:\n- \"reqwest-rustls-tls\"\n- \"reqwest-native-tls\"\n");

#[cfg(not(any(feature = "rustcrypto", feature = "openssl", feature = "wincrypto")))]
cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        compile_error!("One of the following features MUST be enabled:\n- \"rustcrypto\"\n- \"openssl\"\n- \"wincrypto\"\n");
    } else {
        compile_error!("One of the following features MUST be enabled:\n- \"rustcrypto\"\n- \"openssl\"\n");
    }
}

#[cfg(all(feature = "wincrypto", not(target_os = "windows")))]
compile_error!("Feature \"wincrypto\" will only work on Windows!");

mod rsa;
mod aes;
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
