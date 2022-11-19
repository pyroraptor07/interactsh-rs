use interactsh_rs::prelude::*;

use super::utils::{public_utils, shared_utils};


pub async fn client_registers_and_deregisters_to_pub_servers_successfully() {
    let client = public_utils::try_register_to_any_of_pub_servers().await;

    client
        .deregister()
        .await
        .expect("Failed to deregister with the public server");
}


pub async fn client_polls_pub_servers_successfully() {
    let client = public_utils::try_register_to_any_of_pub_servers().await;

    let _log_data = client
        .poll()
        .await
        .expect("Failed to poll the public server");

    client
        .deregister()
        .await
        .expect("Failed to deregister with the public server");
}


pub async fn client_receives_http_logs_from_pub_servers() {
    let client = public_utils::try_register_to_any_of_pub_servers().await;

    let interaction_fqdn = client.get_interaction_fqdn();
    shared_utils::generate_http_interaction(interaction_fqdn, None, None).await;

    let log_data = client
        .poll()
        .await
        .expect("Failed to poll the public server");

    let log_entries = match log_data {
        Some(log_entries) => log_entries,
        None => panic!("No logs recieved from public server"),
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

    panic!("No HTTP logs recieved from public server");
}


pub async fn client_receives_dns_logs_from_pub_servers() {
    let client = public_utils::try_register_to_any_of_pub_servers().await;

    let interaction_fqdn = client.get_interaction_fqdn();
    shared_utils::generate_dns_interaction(interaction_fqdn).await;

    let log_data = client
        .poll()
        .await
        .expect("Failed to poll the public server");

    let log_entries = match log_data {
        Some(log_entries) => log_entries,
        None => panic!("No logs recieved from public server"),
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

    panic!("No DNS logs recieved from public server");
}
