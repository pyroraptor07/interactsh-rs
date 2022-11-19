//! Contains the primary structures necessary for registering and polling
//! an Interactsh server.
//!
//! In order to start a session with an Interactsh server, you must first
//! build an [UnregisteredClient] using a [ClientBuilder]. Then, call the
//! [register](UnregisteredClient::register()) function on the client to
//! turn it into a [RegisteredClient] that can be used to poll the server.
//! Once you are done with the client, you should call the
//! [deregister](RegisteredClient::deregister()) function on the client
//! to deregister with the Interactsh server and dispose of the client.
//!
//! The [poll](RegisteredClient::poll()) function can be used to poll the server
//! once successfully registered. If there are new logs, they will be
//! returned as a vec of [LogEntry](crate::interaction_log::LogEntry) (see the LogEntry page for more
//! details).
//!
//! ```
//! use std::time::Duration;
//! # use std::thread;
//! use interactsh_rs::prelude::*;
//!
//! async fn run() {
//!     // When using ClientBuilder::new(), make sure to set
//!     // all required options (server and rsa key size).
//!     let mut builder = ClientBuilder::new()
//!         .with_server("oast.pro".into())
//!         .with_rsa_key_size(2048);
//!
//!     // These settings are optional or have default values
//!     builder = builder
//!         .with_auth_token("some-token-string".into()) // optional
//!         .with_timeout(Duration::from_secs(20)) // defaults to 15 seconds
//!         .verify_ssl(true) // defaults to false
//!         .parse_logs(false); // defaults to true
//!
//!     // Build the client, then register with the server.
//!     let unregistered_client = builder
//!         .build()
//!         .expect("Error when building the client");
//!
//!     let registered_client = unregistered_client
//!         .register()
//!         .await
//!         .expect("Error when registering the client");
//!
//!     println!("INTERACTION URL: https://{}", registered_client.get_interaction_fqdn());
//!
//!     // Start a poll loop
//!     loop {
//!         // --- (omitted) sleep and check if loop should break ---
//!         # thread::sleep(Duration::from_secs(5));
//!         # if should_end() { break; }
//!
//!         // Poll the server for new logs
//!         let poll_data = registered_client
//!             .poll()
//!             .await
//!             .expect("Error when polling the server");
//!
//!         let logs = match poll_data {
//!             Some(logs) => logs,
//!             None => continue,
//!         };
//!
//!         // In the returned vec, each log may be either raw or parsed.
//!         //
//!         // Note: Parsed logs do not implement Display; log formatting
//!         // should be handled by the application using this crate.
//!         for log_entry in logs.iter() {
//!             let output = match log_entry {
//!                 LogEntry::ParsedLog(log) => format_logs(log),
//!                 LogEntry::RawLog(log) => log.log_entry.clone(),
//!             };
//!
//!             println!("LOG:\n{}", output);
//!         }
//!     }
//!
//!     // Deregister the client once completed
//!     registered_client.deregister()
//!         .await
//!         .expect("Error when deregistering the client");
//! }
//!
//! fn format_logs(log_entry: &ParsedLogEntry) -> String {
//!     // --- (omitted) format the parsed logs ---
//! #    format!("{:?}", log_entry)
//! }
//!
//! # fn should_end() -> bool {
//! #    false
//! # }
//! ```

pub(crate) mod http_utils;

mod builder;
// pub mod proxy;
pub(crate) mod errors;
mod registered;
mod unregistered;

pub use builder::*;
pub use registered::*;
pub use unregistered::*;
