mod builder;
mod client;

// External re-exports
pub use reqwest::Proxy;

// Crate re-exports
pub use self::client::InteractshClient;
#[cfg(feature = "log-stream")]
pub use self::client::LogPollResult;
pub use super::client_shared::correlation::CorrelationConfig;
