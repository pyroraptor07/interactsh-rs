use interactsh_rs::client_next::*;

/// The default list of servers provided by the Interactsh team
const DEFAULT_INTERACTSH_SERVERS: &[&str] = &[
    "oast.pro",
    "oast.live",
    "oast.site",
    "oast.online",
    "oast.fun",
    "oast.me",
];

/// Builds a client for the provided public server
pub fn build_pub_client(server: String, proxy: Option<Proxy>) -> InteractshClient {
    let mut builder = ClientBuilder::new()
        .with_rsa_key_size(2048)
        .with_server(server);

    builder = if let Some(proxy) = proxy {
        builder.with_proxy(proxy)
    } else {
        builder
    };

    builder
        .build()
        .expect("Failed to build the client for the public server")
}

/// Trys to register a client with any of the known public servers
/// and returns the first successfully registered client
pub async fn try_register_to_any_of_pub_servers(
    proxy: Option<Proxy>,
) -> (InteractshClient, String) {
    let mut pub_servers = DEFAULT_INTERACTSH_SERVERS.iter();

    while let Some(server) = pub_servers.next() {
        let client = build_pub_client(server.to_string(), proxy.clone());
        let register_result = client.register().await;

        if let Ok(fqdn) = register_result {
            return (client, fqdn);
        }
    }

    panic!("Unable to register to any public server");
}
