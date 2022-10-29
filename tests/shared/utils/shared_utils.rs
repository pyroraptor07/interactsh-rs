use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use async_compat::Compat;
use trust_dns_resolver::config::*;
use trust_dns_resolver::AsyncResolver;


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
