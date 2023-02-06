#[cfg(feature = "async-compat")]
mod shared;

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn client_registers_and_deregisters_to_pub_servers_successfully() {
    shared::client_registers_and_deregisters_to_pub_servers_successfully().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn client_polls_pub_servers_successfully() {
    shared::client_polls_pub_servers_successfully().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn client_receives_http_logs_from_pub_servers() {
    shared::client_receives_http_logs_from_pub_servers().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn client_receives_http_logs_from_proxied_pub_servers() {
    shared::client_receives_http_logs_from_proxied_pub_servers().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
#[ignore] // When run in Github Actions, DNS interaction tests intermittently fail
async fn client_receives_dns_logs_from_pub_servers() {
    shared::client_receives_dns_logs_from_pub_servers().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn client_registers_and_deregisters_to_local_server_successfully_with_auth() {
    shared::client_registers_and_deregisters_to_local_server_successfully_with_auth().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn client_polls_local_server_successfully() {
    shared::client_polls_local_server_successfully().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn client_receives_http_logs_from_local_server() {
    shared::client_receives_http_logs_from_local_server().await;
}


// ++++++++++++++++++++ NEW CLIENT +++++++++++++++++++++++++++++

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn new_client_registers_and_deregisters_to_pub_servers_successfully() {
    shared::public_tests_new_client::client_registers_and_deregisters_to_pub_servers_successfully()
        .await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn new_client_polls_pub_servers_successfully() {
    shared::public_tests_new_client::client_polls_pub_servers_successfully().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn new_client_receives_http_logs_from_pub_servers() {
    shared::public_tests_new_client::client_receives_http_logs_from_pub_servers().await;
}

#[cfg(all(feature = "async-compat", feature = "log-stream"))]
#[async_std::test]
async fn log_stream_receives_http_logs_from_pub_servers() {
    shared::public_tests_new_client::log_stream_receives_http_logs_from_pub_servers().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
async fn new_client_receives_http_logs_from_proxied_pub_servers() {
    shared::public_tests_new_client::client_receives_http_logs_from_proxied_pub_servers().await;
}

#[cfg(feature = "async-compat")]
#[async_std::test]
#[ignore] // When run in Github Actions, DNS interaction tests intermittently fail
async fn new_client_receives_dns_logs_from_pub_servers() {
    shared::public_tests_new_client::client_receives_dns_logs_from_pub_servers().await;
}
