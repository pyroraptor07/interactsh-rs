

fn main() {
    if enable_nightly() {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
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
