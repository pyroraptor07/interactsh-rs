use std::sync::Arc;

use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;

use super::server_comm::InteractionFqdn;
use crate::client_shared::server_comm::ClientStatus;


#[derive(Debug)]
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
    pub(crate) subdomain: String,
    pub(crate) correlation_id: String,
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

    pub fn into_client_status(self, server: String) -> (ClientStatus, Arc<InteractionFqdn>) {
        let interaction_fqdn = Arc::new(InteractionFqdn {
            subdomain: self.subdomain,
            server,
        });

        let fqdn_ref = Arc::clone(&interaction_fqdn);

        let status = ClientStatus::Registered {
            interaction_fqdn,
            correlation_id: self.correlation_id,
        };

        (status, fqdn_ref)
    }
}

impl Default for CorrelationData {
    fn default() -> Self {
        Self::generate_data(&CorrelationConfig::default())
    }
}
