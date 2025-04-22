# Interactsh-rs

![Maintenance](https://img.shields.io/maintenance/no/2025)
[![Documentation](https://docs.rs/interactsh-rs/badge.svg)](https://docs.rs/interactsh-rs)
[![Crates.io](https://img.shields.io/crates/v/interactsh-rs.svg)](https://crates.io/crates/interactsh-rs)
[![License](https://img.shields.io/crates/l/interactsh-rs.svg)](https://github.com/pyroraptor07/interactsh-rs)
[![Downloads](https://img.shields.io/crates/d/interactsh-rs.svg)](https://crates.io/crates/interactsh-rs)
[![CI Tests](https://github.com/pyroraptor07/interactsh-rs/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/pyroraptor07/interactsh-rs/actions/workflows/main.yml)

A Rust client library for getting interaction logs from [Interactsh](https://github.com/projectdiscovery/interactsh) servers. See a basic example below; check out the [examples](https://github.com/pyroraptor07/interactsh-rs/tree/main/examples) or the client module in the [API docs](https://docs.rs/interactsh-rs/latest/interactsh_rs/client/index.html) for more detailed use.

## !! 2025 MAINTENANCE STATUS UPDATE !!
Unfortunately, I no longer have the time nor motivation to upkeep this crate. If anyone would like to take up maintenance of this crate, please let me know and I will transfer ownership to you. Feel free to fork this crate as well. I do not recommend using this crate as is; the dependancies need to be updated and the crate itself honestly needs a rewrite (at the very least to be more generic over crypto implementations).

## Basic Use Example
```rust
use std::time::Duration;
use std::thread;
use interactsh_rs::prelude::*;

async fn run_client() {
    // Builds an unregistered client
    let client = ClientBuilder::default()
        .with_server("oast.pro".into())
        .parse_logs(true)
        .build()
        .unwrap();

    // Registers the client with the server and
    // returns a registered client
    let client = client.register().await.unwrap();
    let interaction_fqdn = client.get_interaction_fqdn();
    println!("INTERACTION URL: https://{}", interaction_fqdn);

    // Start a poll loop
    loop {
        thread::sleep(Duration::from_secs(5));

        let logs = match client.poll().await.unwrap() {
            Some(logs) => logs,
            None => continue,
        };

        // ...Do something with the returned logs...
    }

    // Once done, deregister the client
    client.deregister().await.unwrap();
}
```
