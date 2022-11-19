use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use async_compat::Compat;
use once_cell::sync::Lazy;
use trust_dns_resolver::config::*;
use trust_dns_resolver::AsyncResolver;


/// Loads local server data from environment
static TEST_ENV_SETTINGS: Lazy<TestSettings> = Lazy::new(|| {
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

    let proxy_server_addr = dotenvy::var("INTERACTSHRS_TEST_PROXY_ADDR")
        .expect("Environment variable 'INTERACTSHRS_TEST_PROXY_ADDR' is missing");

    let proxy_server_port = dotenvy::var("INTERACTSHRS_TEST_PROXY_PORT")
        .expect("Environment variable 'INTERACTSHRS_TEST_PROXY_PORT' is missing");

    let local_server = LocalServer {
        server_fqdn,
        auth_token,
        dns_override_addr,
    };

    let proxy_server_url = format!("http://{}:{}", proxy_server_addr, proxy_server_port);

    let proxy_server = reqwest::Proxy::all(proxy_server_url).expect("Invalid proxy settings");

    TestSettings {
        local_server,
        proxy_server,
    }
});


/// Container for the test settings
struct TestSettings {
    local_server: LocalServer,
    proxy_server: reqwest::Proxy,
}


/// Container for the local server data
#[derive(Clone)]
pub struct LocalServer {
    pub server_fqdn: String,
    pub auth_token: String,
    pub dns_override_addr: Option<Ipv4Addr>,
}

pub fn get_local_server() -> LocalServer {
    Lazy::force(&TEST_ENV_SETTINGS).local_server.clone()
}

pub fn get_proxy() -> reqwest::Proxy {
    Lazy::force(&TEST_ENV_SETTINGS).proxy_server.clone()
}


/// Generates an http interaction with the provided server
pub async fn generate_http_interaction(
    interaction_fqdn: String,
    token: Option<String>,
    dns_override_addr: Option<Ipv4Addr>,
) {
    let interaction_url = format!("https://{interaction_fqdn}");
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .danger_accept_invalid_certs(true);

    builder = if let Some(override_addr) = dns_override_addr {
        let ip_addr = IpAddr::V4(override_addr);
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

/// Generates a dns interaction with the provided server
///
/// Currently only works with public servers
pub async fn generate_dns_interaction(interaction_fqdn: String) {
    let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default())
        .expect("Failed to create the dns resolver");

    let lookup_future = Compat::new(async { resolver.lookup_ip(interaction_fqdn).await });

    lookup_future.await.expect("DNS request failed");
}
