use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;

use super::client::ClientStatus;

pub struct CorrelationConfig {
    pub subdomain_length: usize,
    pub correlation_id_length: usize,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            subdomain_length: 33,
            correlation_id_length: 20,
        }
    }
}

pub(crate) struct CorrelationData {
    subdomain: String,
    correlation_id: String,
}

impl CorrelationData {
    pub fn generate_data(config: &CorrelationConfig) -> Self {
        let max_length = config.subdomain_length.max(config.correlation_id_length);
        let mut subdomain = Alphanumeric
            .sample_string(&mut thread_rng(), max_length)
            .to_ascii_lowercase();

        let mut correlation_id = subdomain.clone();

        subdomain.truncate(config.subdomain_length);
        correlation_id.truncate(config.correlation_id_length);

        Self {
            subdomain,
            correlation_id,
        }
    }
}

impl Default for CorrelationData {
    fn default() -> Self {
        Self::generate_data(&CorrelationConfig::default())
    }
}

impl From<CorrelationData> for ClientStatus {
    fn from(data: CorrelationData) -> Self {
        ClientStatus::Registered {
            subdomain: data.subdomain,
            correlation_id: data.correlation_id,
        }
    }
}
