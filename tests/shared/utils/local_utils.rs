use std::net::IpAddr;
use std::time::Duration;

use interactsh_rs::prelude::*;

use super::shared_utils::get_local_server;


/// Trys to register a client with the local server provided
/// via environment variables and returns a registered client
/// if successful
pub async fn try_register_to_local_server() -> RegisteredClient {
    let local_server = get_local_server();
    let builder = ClientBuilder::new()
        .with_server(local_server.server_fqdn)
        .with_auth_token(local_server.auth_token)
        .with_rsa_key_size(2048)
        .verify_ssl(false)
        .parse_logs(true)
        .with_timeout(Duration::from_secs(5));

    let builder = if let Some(override_addr) = local_server.dns_override_addr {
        let ip_addr = IpAddr::V4(override_addr);
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
