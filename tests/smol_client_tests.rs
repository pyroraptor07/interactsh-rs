#[cfg(feature = "async-compat")]
mod shared;

#[cfg(feature = "async-compat")]
#[test]
fn client_registers_and_deregisters_to_pub_servers_successfully() {
    smol::block_on(async {
        shared::client_registers_and_deregisters_to_pub_servers_successfully().await;
    });
}

#[cfg(feature = "async-compat")]
#[test]
fn client_polls_pub_servers_successfully() {
    smol::block_on(async {
        shared::client_polls_pub_servers_successfully().await;
    });
}

#[cfg(feature = "async-compat")]
#[test]
fn client_receives_http_logs_from_pub_servers() {
    smol::block_on(async {
        shared::client_receives_http_logs_from_pub_servers().await;
    });
}

#[cfg(feature = "async-compat")]
#[test]
fn client_receives_http_logs_from_proxied_pub_servers() {
    smol::block_on(async {
        shared::client_receives_http_logs_from_proxied_pub_servers().await;
    });
}

#[cfg(feature = "async-compat")]
#[test]
#[ignore] // When run in Github Actions, DNS interaction tests intermittently fail
fn client_receives_dns_logs_from_pub_servers() {
    smol::block_on(async {
        shared::client_receives_dns_logs_from_pub_servers().await;
    });
}

#[cfg(feature = "async-compat")]
#[test]
fn client_registers_and_deregisters_to_local_server_successfully_with_auth() {
    smol::block_on(async {
        shared::client_registers_and_deregisters_to_local_server_successfully_with_auth().await;
    });
}

#[cfg(feature = "async-compat")]
#[test]
fn client_polls_local_server_successfully() {
    smol::block_on(async {
        shared::client_polls_local_server_successfully().await;
    });
}

#[cfg(feature = "async-compat")]
#[test]
fn client_receives_http_logs_from_local_server() {
    smol::block_on(async {
        shared::client_receives_http_logs_from_local_server().await;
    });
}
