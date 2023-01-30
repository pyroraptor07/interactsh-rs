// use interactsh_rs::client_next::LogPollResult;
// use interactsh_rs::futures_util::StreamExt;
use interactsh_rs::interaction_log::*;

// use snafu::ResultExt;
use super::utils::{public_utils_new_client, shared_utils};


pub async fn client_registers_and_deregisters_to_pub_servers_successfully() {
    let client = public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

    client
        .deregister()
        .await
        .expect("Failed to deregister with the public server");
}


pub async fn client_polls_pub_servers_successfully() {
    let client = public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

    let _log_data = client
        .poll()
        .await
        .expect("Failed to poll the public server");

    client
        .deregister()
        .await
        .expect("Failed to deregister with the public server");
}


// pub async fn log_stream_receives_http_logs_from_pub_servers() {
//     let client = public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

//     let interaction_fqdn = client
//         .get_interaction_fqdn()
//         .expect("Client is not registered, no fqdn returned");
//     shared_utils::generate_http_interaction(interaction_fqdn, None, None).await;

//     let log_stream = client.log_stream_map_filter(std::time::Duration::from_secs(1), |res| {
//         match res {
//             LogPollResult::NoNewLogs => {
//                 Some(Err("No new logs recieved").whatever_context("No new logs recieved"))
//             }
//             LogPollResult::ReceivedNewLog(log) => Some(Ok(log)),
//             LogPollResult::Error(e) => Some(Err(e)),
//         }
//     });
//     let mut boxed_log_stream = Box::pin(log_stream);

//     while let Some(Ok(log_entry)) = boxed_log_stream.next().await {
//         match log_entry {
//             LogEntry::ParsedLog(ParsedLogEntry::Http { .. }) => {
//                 client
//                     .deregister()
//                     .await
//                     .expect("Failed to deregister with the public server");
//                 return;
//             }
//             _ => continue,
//         }
//     }

//     panic!("No HTTP logs recieved from public server");
// }


pub async fn client_receives_http_logs_from_pub_servers() {
    let client = public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

    let interaction_fqdn = client
        .get_interaction_fqdn()
        .expect("Client is not registered, no fqdn returned");
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


pub async fn client_receives_http_logs_from_proxied_pub_servers() {
    let proxy_server = shared_utils::get_proxy();
    let client =
        public_utils_new_client::try_register_to_any_of_pub_servers(Some(proxy_server)).await;

    let interaction_fqdn = client
        .get_interaction_fqdn()
        .expect("Client is not registered, no fqdn returned");
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
    let client = public_utils_new_client::try_register_to_any_of_pub_servers(None).await;

    let interaction_fqdn = client
        .get_interaction_fqdn()
        .expect("Client is not registered, no fqdn returned");
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
