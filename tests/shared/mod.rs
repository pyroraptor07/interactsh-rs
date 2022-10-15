use std::time::Duration;

use async_compat::Compat;
use interactsh_rs::prelude::*;
use trust_dns_resolver::config::*;
use trust_dns_resolver::AsyncResolver;


/// The default list of servers provided by the Interactsh team
const DEFAULT_INTERACTSH_SERVERS: &[&str] = &[
    "oast.pro",
    "oast.live",
    "oast.site",
    "oast.online",
    "oast.fun",
    "oast.me",
];

fn build_pub_client(server: String) -> UnregisteredClient {
    ClientBuilder::new()
        .with_rsa_key_size(2048)
        .with_server(server)
        .build()
        .expect("Failed to build the client")
}

async fn try_register_to_any_of_pub_servers() -> RegisteredClient {
    let mut pub_servers = DEFAULT_INTERACTSH_SERVERS.iter();

    while let Some(server) = pub_servers.next() {
        let unregistered_client = build_pub_client(server.to_string());
        let register_result = unregistered_client.register().await;

        if let Ok(registered_client) = register_result {
            return registered_client;
        }
    }

    panic!("Unable to register to any public server");
}

async fn generate_http_interaction(interaction_fqdn: String, token: Option<String>) {
    let interaction_url = format!("https://{interaction_fqdn}");
    let reqwest_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build the reqwest client for HTTP interaction");

    let get_future = Compat::new(async {
        let mut get_request = reqwest_client.get(interaction_url);

        get_request = match token {
            Some(token) => get_request.header("Authorization", token),
            None => get_request,
        };

        get_request.send().await
    });
    let get_response = get_future.await.expect("Failed to send HTTP get request");

    if !get_response.status().is_success() {
        panic!("HTTP get request did not get success status back");
    }
}

async fn generate_dns_interaction(interaction_fqdn: String) {
    let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default())
        .expect("Failed to create the dns resolver");

    let lookup_future = Compat::new(async { resolver.lookup_ip(interaction_fqdn).await });

    lookup_future.await.expect("DNS request failed");
}

pub async fn client_registers_and_deregisters_to_pub_servers_successfully() {
    let client = try_register_to_any_of_pub_servers().await;

    client
        .deregister()
        .await
        .expect("Failed to deregister with the public server");
}

pub async fn client_polls_pub_servers_successfully() {
    let client = try_register_to_any_of_pub_servers().await;

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
    let client = try_register_to_any_of_pub_servers().await;

    let interaction_fqdn = client.get_interaction_url();
    generate_http_interaction(interaction_fqdn, None).await;

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
    let client = try_register_to_any_of_pub_servers().await;

    let interaction_fqdn = client.get_interaction_url();
    generate_dns_interaction(interaction_fqdn).await;

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

pub async fn client_registers_and_deregisters_to_local_server_successfully_with_auth() {
    todo!()
}

pub async fn client_polls_local_server_successfully() {
    todo!()
}

pub async fn client_receives_http_logs_from_local_server() {
    todo!()
}
