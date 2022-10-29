fn main() {
    if enable_nightly() {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }

    if cfg!(all(feature = "rustcrypto", feature = "openssl")) {
        println!("cargo:warning=Both 'rustcrypto' and 'openssl' features are enabled - defaulting to 'rustcrypto'");
    }

    if cfg!(all(feature = "rustls-tls", feature = "native-tls")) {
        println!("cargo:warning=Both 'rustls-tls' and 'native-tls' features are enabled - defaulting to 'rustls-tls'");
    }
}

#[rustversion::nightly]
fn enable_nightly() -> bool {
    true
}

#[rustversion::not(nightly)]
fn enable_nightly() -> bool {
    false
}
