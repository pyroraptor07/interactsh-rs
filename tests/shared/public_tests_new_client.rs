#[cfg(feature = "log-stream")]
use interactsh_rs::client_next::LogPollResult;
#[cfg(feature = "log-stream")]
use interactsh_rs::futures_util::StreamExt;
use interactsh_rs::interaction_log::*;

use super::utils::{public_utils_new_client, shared_utils};


pub async fn client_registers_and_deregisters_to_pub_servers_successfully() {
    let (client, _) = public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

    client
        .deregister()
        .await
        .expect("Failed to deregister with the public server");
}


pub async fn client_polls_pub_servers_successfully() {
    let (client, _) = public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

    let _log_data = client
        .poll()
        .await
        .expect("Failed to poll the public server");

    client
        .deregister()
        .await
        .expect("Failed to deregister with the public server");
}

#[cfg(feature = "log-stream")]
pub async fn log_stream_receives_http_logs_from_pub_servers() {
    let (client, interaction_fqdn) =
        public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

    shared_utils::generate_http_interaction(interaction_fqdn, None, None).await;

    let mut log_stream = client.log_stream_filter_map(std::time::Duration::from_secs(1), |res| {
        match res {
            LogPollResult::NoNewLogs => Some(Err("No new logs received")),
            LogPollResult::ReceivedNewLog(log) => Some(Ok(log)),
            LogPollResult::Error(_) => Some(Err("Poll error")),
        }
    });

    while let Some(Ok(log_entry)) = log_stream.next().await {
        match log_entry {
            LogEntry::ParsedLog(ParsedLogEntry::Http { .. }) => {
                client
                    .deregister()
                    .await
                    .expect("Failed to deregister with the public server");
                return;
            }
            _ => continue,
        }
    }

    panic!("No HTTP logs received from public server");
}


pub async fn client_receives_http_logs_from_pub_servers() {
    let (client, interaction_fqdn) =
        public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

    shared_utils::generate_http_interaction(interaction_fqdn, None, None).await;

    let log_data = client
        .poll()
        .await
        .expect("Failed to poll the public server");

    let log_entries = match log_data {
        Some(log_entries) => log_entries,
        None => panic!("No logs received from public server"),
    };

    let mut log_entries = log_entries.into_iter();
    while let Some(log_entry) = log_entries.next() {
        match log_entry {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Http { .. } => {
                        client
                            .deregister()
                            .await
                            .expect("Failed to deregister with the public server");

                        return;
                    }
                    _ => continue,
                }
            }
            LogEntry::RawLog(_) => continue,
        }
    }

    panic!("No HTTP logs received from public server");
}


pub async fn client_receives_http_logs_from_proxied_pub_servers() {
    let proxy_server = shared_utils::get_proxy();
    let (client, interaction_fqdn) =
        public_utils_new_client::try_register_to_any_of_pub_servers(Some(proxy_server)).await;

    shared_utils::generate_http_interaction(interaction_fqdn, None, None).await;

    let log_data = client
        .poll()
        .await
        .expect("Failed to poll the public server");

    let log_entries = match log_data {
        Some(log_entries) => log_entries,
        None => panic!("No logs received from public server"),
    };

    let mut log_entries = log_entries.into_iter();
    while let Some(log_entry) = log_entries.next() {
        match log_entry {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Http { .. } => {
                        client
                            .deregister()
                            .await
                            .expect("Failed to deregister with the public server");

                        return;
                    }
                    _ => continue,
                }
            }
            LogEntry::RawLog(_) => continue,
        }
    }

    panic!("No HTTP logs received from public server");
}


pub async fn client_receives_dns_logs_from_pub_servers() {
    let (client, interaction_fqdn) =
        public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

    shared_utils::generate_dns_interaction(interaction_fqdn).await;

    let log_data = client
        .poll()
        .await
        .expect("Failed to poll the public server");

    let log_entries = match log_data {
        Some(log_entries) => log_entries,
        None => panic!("No logs received from public server"),
    };

    let mut log_entries = log_entries.into_iter();
    while let Some(log_entry) = log_entries.next() {
        match log_entry {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Dns { .. } => {
                        client
                            .deregister()
                            .await
                            .expect("Failed to deregister with the public server");

                        return;
                    }
                    _ => continue,
                }
            }
            LogEntry::RawLog(_) => continue,
        }
    }

    panic!("No DNS logs received from public server");
}
