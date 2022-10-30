use std::time::Duration;

use clap::Parser;
use cli_client_shared::{
    print_interaction_url,
    start_ctrlc_listener,
    start_spinner,
    ClientCli,
    LogDisplay,
};
use color_eyre::Result;
use interactsh_rs::prelude::*;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    if let Err(e) = color_eyre::install() {
        eprintln!("Error: {:?}", e);
    }

    if let Err(e) = run_cli_client().await {
        eprintln!("Error: {:?}", e);
    }
}

async fn run_cli_client() -> Result<()> {
    // Build the client
    let client = build_client()?;

    // Register the client
    let spinner = start_spinner("Registering the client...".to_owned());
    let client = client.register().await?;
    spinner.finish_with_message("Client registered successfully!");

    print_interaction_url(client.get_interaction_url());
    let shutdown_rx = start_ctrlc_listener();

    // Poll server
    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("Polling the server...");
    poll_server(&client, shutdown_rx).await?;

    // Deregister the client
    let spinner = start_spinner("Deregistering the client...".to_owned());
    client.deregister().await?;
    spinner.finish_with_message("Client deregistered successfully!");

    Ok(())
}

fn build_client() -> Result<UnregisteredClient> {
    let builder = get_builder_from_cli();

    let spinner = start_spinner("Building the client...".to_owned());
    let client = builder.build()?;
    spinner.finish_with_message("Client built successfully!");

    Ok(client)
}

fn get_builder_from_cli() -> ClientBuilder {
    let cli = ClientCli::parse();
    let mut builder = ClientBuilder::default();

    builder = if let Some(server) = cli.server {
        builder.with_server(server)
    } else {
        builder
    };

    builder = if let Some(token) = cli.auth_token {
        builder.with_auth_token(token)
    } else {
        builder
    };

    builder = if let Some(key_size) = cli.key_size {
        builder.with_rsa_key_size(key_size)
    } else {
        builder
    };

    builder = if let Some(timeout) = cli.timeout {
        builder.with_timeout(Duration::from_secs(timeout))
    } else {
        builder
    };

    builder = builder.verify_ssl(cli.ssl_verify);
    builder = builder.parse_logs(!cli.raw_logs);

    builder
}

async fn poll_server(client: &RegisteredClient, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
    tokio::select! {
        _ = shutdown_rx => return Ok(()),
        result = poll_loop(client) => return result,
    }
}

async fn poll_loop(client: &RegisteredClient) -> Result<()> {
    loop {
        let logs = match client.poll().await? {
            Some(logs) => logs,
            None => continue,
        };

        for log_entry in logs.iter() {
            match log_entry {
                LogEntry::ParsedLog(log) => println!("{}", log.as_formatted_log_string()),
                LogEntry::RawLog(log) => println!("{}", log.as_formatted_log_string()),
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
