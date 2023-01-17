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
//!     let interaction_fqdn = client.get_interaction_fqdn();
//!     println!("INTERACTION URL: https://{}", interaction_fqdn);
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
//! - `rustls-tls`
//! - `native-tls`
//! - `native-tls-vendored`
//!
//! One of these must be enabled as well to use the crate as a client.
//! `rustls-tls` is enabled by default.
//!
//! Note: All 3 TLS feature flags can also be used currently with the "reqwest-"
//! prefix. These were the original TLS feature flag names used in initial
//! development, but will be removed in a future release in favor of the shorter
//! feature names omitting the "reqwest-" prefix.
//!
//! ## Feature Flags - Async runtime compatibility
//! This crate supports the [tokio](https://github.com/tokio-rs/tokio),
//! [async-std](https://github.com/async-rs/async-std), and
//! [smol](https://github.com/smol-rs/smol) async runtimes. In order to use
//! non-tokio runtimes with this crate, use the `async-compat` feature flag
//! (enabled by default).

#![cfg_attr(feature = "nightly", feature(doc_auto_cfg))]

#[cfg(any(feature = "rustcrypto", feature = "openssl"))]
mod crypto;

#[cfg(all(
    any(feature = "rustls-tls", feature = "native-tls"),
    any(feature = "rustcrypto", feature = "openssl")
))]
pub mod client;
#[cfg(all(
    any(feature = "rustls-tls", feature = "native-tls"),
    any(feature = "rustcrypto", feature = "openssl"),
    feature = "client-next",
))]
pub mod client_next;
#[cfg(all(
    any(feature = "rustls-tls", feature = "native-tls"),
    any(feature = "rustcrypto", feature = "openssl")
))]
mod client_shared;
#[cfg(all(
    any(feature = "rustls-tls", feature = "native-tls"),
    any(feature = "rustcrypto", feature = "openssl")
))]
pub mod errors;
pub mod interaction_log;

// re-exports
#[cfg(feature = "log-stream")]
pub use futures_util;

pub mod prelude {
    #[cfg(all(
        any(feature = "rustls-tls", feature = "native-tls"),
        any(feature = "rustcrypto", feature = "openssl")
    ))]
    pub use crate::client::*;
    #[cfg(all(
        any(feature = "rustls-tls", feature = "native-tls"),
        any(feature = "rustcrypto", feature = "openssl")
    ))]
    pub use crate::errors::*;
    pub use crate::interaction_log::*;
}



//                                                     .--.__
//                                                   .~ (@)  ~~~---_
//                                                  {     `-_~,,,,,,)
//                                                  {    (_  ',
//                                                   ~    . = _',
//                                                    ~-   '.  =-'
//                                                      ~     :
//   .                                             _,.-~     ('');
//   '.                                         .-~        \  \ ;
//     ':-_                                _.--~            \  \;      _-=,.
//       ~-:-.__                       _.-~                 {  '---- _'-=,.
//          ~-._~--._             __.-~                     ~---------=,.`
//              ~~-._~~-----~~~~~~       .+++~~~~~~~~-__   /
//                   ~-.,____           {   -     +   }  _/
//                           ~~-.______{_    _ -=\ / /_.~
//                                :      ~--~    // /         ..-
//                                :   / /      // /         ((
//                                :  / /      {   `-------,. ))
//                                :   /        ''=--------. }o
//                   .=._________,'  )                     ))
//                   )  _________ -''                     ~~
//                  / /  _ _
//                 (_.-.'O'-'.
//
// source: https://ascii.co.uk/art/dinosaur
