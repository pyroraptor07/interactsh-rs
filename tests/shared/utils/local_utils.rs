use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use interactsh_rs::prelude::*;
use once_cell::sync::Lazy;


/// Loads local server data from environment
static LOCAL_SERVER_DATA: Lazy<LocalServer> = Lazy::new(|| {
    dotenvy::dotenv().ok();

    let server_fqdn = dotenvy::var("INTERACTSHRS_TEST_LOCAL_SERVER_FQDN")
        .expect("Environment variable 'INTERACTSHRS_TEST_LOCAL_SERVER_FQDN' is missing");

    let auth_token = dotenvy::var("INTERACTSHRS_TEST_LOCAL_SERVER_TOKEN")
        .expect("Environment variable 'INTERACTSHRS_TEST_LOCAL_SERVER_TOKEN' is missing");

    let dns_override_addr = dotenvy::var("INTERACTSHRS_TEST_LOCAL_SERVER_DNS_OVERRIDE_ADDR")
        .ok()
        .and_then(|env_val| {
            let override_addr = env_val.parse::<Ipv4Addr>();
            override_addr.ok()
        });

    LocalServer {
        server_fqdn,
        auth_token,
        dns_override_addr,
    }
});


/// Container for the local server data
#[derive(Clone)]
pub struct LocalServer {
    pub server_fqdn: String,
    pub auth_token: String,
    pub dns_override_addr: Option<Ipv4Addr>,
}

pub fn get_local_server() -> LocalServer {
    Lazy::force(&LOCAL_SERVER_DATA).clone()
}


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
