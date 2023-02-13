mod builder;
mod client;

// External re-exports
pub use reqwest::Proxy;

// Crate re-exports
pub use self::builder::ClientBuilder;
pub use self::client::InteractshClient;
#[cfg(feature = "log-stream")]
pub use self::client::LogPollResult;
pub use super::client_shared::correlation::CorrelationConfig;
pub use super::client_shared::errors::*;

// client_next prelude
pub mod prelude {
    #[cfg(feature = "log-stream")]
    pub use super::LogPollResult;
    pub use super::{
        ClientBuildError,
        ClientBuilder,
        ClientError,
        CorrelationConfig,
        InteractshClient,
        PollError,
        Proxy,
        RegistrationError,
    };
}
