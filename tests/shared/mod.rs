use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use async_compat::Compat;
use interactsh_rs::prelude::*;
use once_cell::sync::Lazy;
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

static LOCAL_SERVER_DATA: Lazy<LocalServer> = Lazy::new(|| {
    dotenvy::dotenv().ok();

    let server_fqdn = dotenvy::var("INTERACTSHRS_TEST_LOCAL_SERVER_FQDN")
        .expect("Environment variable 'INTERACTSHRS_TEST_LOCAL_SERVER_FQDN' is missing");

    let auth_token = dotenvy::var("INTERACTSHRS_TEST_LOCAL_SERVER_TOKEN")
        .expect("Environment variable 'INTERACTSHRS_TEST_LOCAL_SERVER_TOKEN' is missing");

    let dns_override = dotenvy::var("INTERACTSHRS_TEST_LOCAL_SERVER_DNS_OVERRIDE")
        .ok()
        .and_then(|env_val| {
            if env_val.eq_ignore_ascii_case("false") {
                None
            } else if env_val.eq_ignore_ascii_case("0") {
                None
            } else {
                Some(())
            }
        })
        .is_some();

    LocalServer {
        server_fqdn,
        auth_token,
        dns_override,
    }
});


#[derive(Clone)]
struct LocalServer {
    server_fqdn: String,
    auth_token: String,
    dns_override: bool,
}

fn get_local_server() -> LocalServer {
    Lazy::force(&LOCAL_SERVER_DATA).clone()
}

fn build_pub_client(server: String) -> UnregisteredClient {
    ClientBuilder::new()
        .with_rsa_key_size(2048)
        .with_server(server)
        .build()
        .expect("Failed to build the client for the public server")
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

async fn try_register_to_local_server() -> RegisteredClient {
    let local_server = get_local_server();
    let builder = ClientBuilder::new()
        .with_server(local_server.server_fqdn)
        .with_auth_token(local_server.auth_token)
        .with_rsa_key_size(2048)
        .verify_ssl(false)
        .parse_logs(true)
        .with_timeout(Duration::from_secs(5));

    let builder = if local_server.dns_override {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        builder.set_dns_override(ip_addr)
    } else {
        builder
    };

    let unregistered_client = builder
        .build()
        .expect("Failed to build the client for the local server");

    unregistered_client
        .register()
        .await
        .expect("Failed to register with the local server")
}

async fn generate_http_interaction(
    interaction_fqdn: String,
    token: Option<String>,
    dns_override: bool,
) {
    let interaction_url = format!("https://{interaction_fqdn}");
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(true);

    builder = if dns_override {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        let socket_addr = SocketAddr::new(ip_addr, 443);
        builder.resolve(interaction_fqdn.as_str(), socket_addr)
    } else {
        builder
    };

    let reqwest_client = builder
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
    generate_http_interaction(interaction_fqdn, None, false).await;

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
    let client = try_register_to_local_server().await;

    client
        .deregister()
        .await
        .expect("Failed to deregister with the local server");
}

pub async fn client_polls_local_server_successfully() {
    let client = try_register_to_local_server().await;

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
    let client = try_register_to_local_server().await;
    let local_server = get_local_server();

    let interaction_fqdn = client.get_interaction_url();
    generate_http_interaction(
        interaction_fqdn,
        Some(local_server.auth_token),
        local_server.dns_override,
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
