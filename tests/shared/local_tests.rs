use interactsh_rs::prelude::*;

use super::utils::{local_utils, shared_utils};


pub async fn client_registers_and_deregisters_to_local_server_successfully_with_auth() {
    let client = local_utils::try_register_to_local_server().await;

    client
        .deregister()
        .await
        .expect("Failed to deregister with the local server");
}


pub async fn client_polls_local_server_successfully() {
    let client = local_utils::try_register_to_local_server().await;

    let _log_data = client
        .poll()
        .await
        .expect("Failed to poll the local server");

    client
        .deregister()
        .await
        .expect("Failed to deregister with the local server");
}


pub async fn client_receives_http_logs_from_local_server() {
    let client = local_utils::try_register_to_local_server().await;
    let local_server = shared_utils::get_local_server();

    let interaction_fqdn = client.get_interaction_fqdn();
    shared_utils::generate_http_interaction(
        interaction_fqdn,
        Some(local_server.auth_token),
        local_server.dns_override_addr,
    )
    .await;

    let log_data = client
        .poll()
        .await
        .expect("Failed to poll the local server");

    let log_entries = match log_data {
        Some(log_entries) => log_entries,
        None => panic!("No logs recieved from local server"),
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
                            .expect("Failed to deregister with the local server");

                        return;
                    }
                    _ => continue,
                }
            }
            LogEntry::RawLog(_) => continue,
        }
    }

    panic!("No HTTP logs recieved from local server");
}
