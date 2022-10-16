//! # Interactsh-rs
//! A Rust client library for getting interaction logs from
//! [Interactsh](https://github.com/projectdiscovery/interactsh) servers. See
//! a basic example below; check out the [examples](https://github.com/pyroraptor07/interactsh-rs/tree/main/examples)
//! or the [client] module docs for more detailed use.
//!
//! ## Basic Use
//! ```
//! use std::time::Duration;
//! use std::thread;
//! use interactsh_rs::prelude::*;
//!
//! async fn run_client() {
//!     // Builds an unregistered client
//!     let client = ClientBuilder::default()
//!         .with_server("oast.pro".into())
//!         .parse_logs(true)
//!         .build()
//!         .unwrap();
//!
//!     // Registers the client with the server and
//!     // returns a registered client
//!     let client = client.register().await.unwrap();
//!     let interaction_url = client.get_interaction_url();
//!     println!("INTERACTION URL: {}", interaction_url);
//!
//!     // Start a poll loop
//!     loop {
//!         thread::sleep(Duration::from_secs(5));
//!
//!         let logs = match client.poll().await.unwrap() {
//!             Some(logs) => logs,
//!             None => continue,
//!         };
//!
//!         // ...Do something with the returned logs...
//!     }
//!
//!     // Once done, deregister the client
//!     client.deregister().await.unwrap();
//! }
//! ```
//!
//! ## Feature Flags - Cryptography
//! This crate supports using either the [RustCrypto](https://github.com/RustCrypto) libraries
//! or OpenSSL for decrypting server logs:
//! - `rustcrypto`
//! - `openssl`
//! - `openssl-vendored`
//!
//! One of these must be enabled in order to use the crate
//! (unless you just need the [interaction_log] module). `rustcrypto` is enabled
//! by default.
//!
//! ## Feature Flags - TLS
//! To enable either [Rustls](https://github.com/rustls/rustls) or OS native
//! TLS, use one of the following feature flags:
//! - `reqwest-rustls-tls`
//! - `reqwest-native-tls`
//! - `reqwest-native-tls-vendored`
//!
//! One of these must be enabled as well to use the crate as a client.
//! `reqwest-rustls-tls` is enabled by default.
//!
//! ## Feature Flags - Async runtime compatibility
//! This crate supports the [tokio](https://github.com/tokio-rs/tokio),
//! [async-std](https://github.com/async-rs/async-std), and
//! [smol](https://github.com/smol-rs/smol) async runtimes. In order to use
//! non-tokio runtimes with this crate, use the `async-compat` feature flag
//! (enabled by default).

#![cfg_attr(feature = "nightly", feature(doc_auto_cfg))]
#![cfg_attr(feature = "nightly", feature(error_generic_member_access))]
#![cfg_attr(feature = "nightly", feature(provide_any))]

#[cfg(any(feature = "rustcrypto", feature = "openssl"))]
pub(crate) mod crypto;

#[cfg(any(feature = "reqwest-rustls-tls", feature = "reqwest-native-tls"))]
pub mod client;
pub mod errors;
pub mod interaction_log;

pub mod prelude {
    #[cfg(any(feature = "reqwest-rustls-tls", feature = "reqwest-native-tls"))]
    pub use crate::client::*;
    pub use crate::interaction_log::*;
}
