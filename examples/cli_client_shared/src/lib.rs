use std::time::Duration;

use clap::Parser;
#[cfg(feature = "non-tokio")]
use color_eyre::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use interactsh_rs::prelude::{ParsedLogEntry, RawLog};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
#[cfg(feature = "tokio")]
use tokio::sync::oneshot;

#[derive(Parser)]
#[clap(author, version, about = "An example CLI client using Interactsh-rs", long_about = None)]
pub struct ClientCli {
    /// Set the Interactsh server to connect to
    #[clap(short = 's', long)]
    pub server: Option<String>,

    /// Set an auth token to use for the server
    #[clap(short = 'a', long)]
    pub auth_token: Option<String>,

    /// Set the RSA key size in bits
    #[clap(short = 'k', long = "key-size")]
    pub key_size: Option<usize>,

    /// Set the request timeout in seconds
    #[clap(short = 't', long)]
    pub timeout: Option<u64>,

    /// Enable SSL certificate verification
    #[clap(short = 'v', long = "verify-ssl")]
    pub ssl_verify: bool,

    /// Output raw logs instead of parsed logs
    #[clap(short = 'r', long = "raw-logs")]
    pub raw_logs: bool,
}

pub fn start_spinner(msg: String) -> ProgressBar {
    let spinner_style = ProgressStyle::with_template("{spinner} [{elapsed}] {msg}")
        .expect("Invalid ProgressStyle template!");
    let spinner = ProgressBar::new_spinner()
        .with_message(msg)
        .with_style(spinner_style);
    spinner.tick();
    spinner.enable_steady_tick(Duration::from_millis(50));

    spinner
}

#[cfg(feature = "non-tokio")]
pub fn start_ctrlc_listener() -> Result<async_channel::Receiver<()>> {
    let (shutdown_tx, shutdown_rx) = async_channel::bounded::<()>(1);
    let shutdown_handler = move || {
        shutdown_tx
            .try_send(())
            .expect("Error sending shutdown signal");
    };

    ctrlc::set_handler(shutdown_handler)?;

    Ok(shutdown_rx)
}

#[cfg(feature = "tokio")]
pub fn start_ctrlc_listener() -> oneshot::Receiver<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let shutdown_handler = async move {
        if let Err(_) = tokio::signal::ctrl_c().await {
            eprintln!("Ctrl C error!");
        }

        if let Err(_) = shutdown_tx.send(()) {
            panic!("Graceful shutdown failed!");
        }
    };

    tokio::spawn(shutdown_handler);

    shutdown_rx
}

pub fn print_interaction_url(interaction_fqdn: String) {
    let interaction_url = format!("https://{}", interaction_fqdn);
    println!(
        "{} {}",
        style("Interaction URL:").bold().green(),
        style(interaction_url).green()
    );
}

pub trait LogDisplay {
    fn as_formatted_log_string(&self) -> String;
}

impl LogDisplay for RawLog {
    fn as_formatted_log_string(&self) -> String {
        format!(
            "{} {}",
            print_log_type("Raw Log"),
            style(self.log_entry.as_str()).blue()
        )
    }
}

impl LogDisplay for ParsedLogEntry {
    fn as_formatted_log_string(&self) -> String {
        match self {
            ParsedLogEntry::Dns {
                unique_id: _,
                full_id,
                q_type,
                raw_request,
                raw_response,
                remote_address,
                timestamp,
            } => {
                match q_type {
                    Some(dns_q_type) => {
                        format!(
                            "{log_type}\n{id}\n{q_type}\n{raw_req}\n{raw_res}\n{addr}\n{timestamp}\n",
                            log_type = print_log_type("DNS Log"),
                            id = print_normal_param("ID", full_id.as_str()),
                            q_type = print_normal_param("Q Type", dns_q_type.to_string().as_str()),
                            raw_req = print_raw_param("Raw Request", raw_request.as_str()),
                            raw_res = print_raw_param("Raw Response", raw_response.as_str()),
                            addr = print_normal_param("Remote Address", remote_address.to_string().as_str()),
                            timestamp = print_timestamp(timestamp),
                        )
                    }
                    None => {
                        format!(
                            "{log_type}\n{id}\n{raw_req}\n{raw_res}\n{addr}\n{timestamp}\n",
                            log_type = print_log_type("DNS Log"),
                            id = print_normal_param("ID", full_id.as_str()),
                            raw_req = print_raw_param("Raw Request", raw_request.as_str()),
                            raw_res = print_raw_param("Raw Response", raw_response.as_str()),
                            addr = print_normal_param(
                                "Remote Address",
                                remote_address.to_string().as_str()
                            ),
                            timestamp = print_timestamp(timestamp),
                        )
                    }
                }
            }
            ParsedLogEntry::Ftp {
                remote_address,
                raw_request,
                timestamp,
            } => {
                format!(
                    "{log_type}\n{addr}\n{raw_req}\n{timestamp}\n",
                    log_type = print_log_type("FTP Log"),
                    addr =
                        print_normal_param("Remote Address", remote_address.to_string().as_str()),
                    raw_req = print_raw_param("Raw Request", raw_request.as_str()),
                    timestamp = print_timestamp(timestamp),
                )
            }
            ParsedLogEntry::Http {
                unique_id: _,
                full_id,
                raw_request,
                raw_response,
                remote_address,
                timestamp,
            } => {
                format!(
                    "{log_type}\n{id}\n{raw_req}\n{raw_res}\n{addr}\n{timestamp}\n",
                    log_type = print_log_type("HTTP Log"),
                    id = print_normal_param("ID", full_id.as_str()),
                    raw_req = print_raw_param("Raw Request", raw_request.as_str()),
                    raw_res = print_raw_param("Raw Response", raw_response.as_str()),
                    addr =
                        print_normal_param("Remote Address", remote_address.to_string().as_str()),
                    timestamp = print_timestamp(timestamp),
                )
            }
            ParsedLogEntry::Ldap {
                unique_id: _,
                full_id,
                raw_request,
                raw_response,
                remote_address,
                timestamp,
            } => {
                format!(
                    "{log_type}\n{id}\n{raw_req}\n{raw_res}\n{addr}\n{timestamp}\n",
                    log_type = print_log_type("LDAP Log"),
                    id = print_normal_param("ID", full_id.as_str()),
                    raw_req = print_raw_param("Raw Request", raw_request.as_str()),
                    raw_res = print_raw_param("Raw Response", raw_response.as_str()),
                    addr =
                        print_normal_param("Remote Address", remote_address.to_string().as_str()),
                    timestamp = print_timestamp(timestamp),
                )
            }
            ParsedLogEntry::Smb {
                raw_request,
                timestamp,
            } => {
                format!(
                    "{log_type}\n{raw_req}\n{timestamp}\n",
                    log_type = print_log_type("SMB Log"),
                    raw_req = print_raw_param("Raw Request", raw_request.as_str()),
                    timestamp = print_timestamp(timestamp),
                )
            }
            ParsedLogEntry::Smtp {
                unique_id: _,
                full_id,
                raw_request,
                smtp_from,
                remote_address,
                timestamp,
            } => {
                format!(
                    "{log_type}\n{id}\n{raw_req}\n{from}\n{addr}\n{timestamp}\n",
                    log_type = print_log_type("SMTP Log"),
                    id = print_normal_param("ID", full_id.as_str()),
                    raw_req = print_raw_param("Raw Request", raw_request.as_str()),
                    from = print_normal_param("SMTP From", smtp_from.as_str()),
                    addr =
                        print_normal_param("Remote Address", remote_address.to_string().as_str()),
                    timestamp = print_timestamp(timestamp),
                )
            }
        }
    }
}

// ParsedLog display helpers
fn print_log_type(log_type: &str) -> String {
    format!(
        "{}{}",
        style(log_type).bold().underlined(),
        style(":").bold().underlined()
    )
}

fn print_normal_param(param: &str, param_data: &str) -> String {
    print_normal_param_nocolor(param, style(param_data).blue().to_string())
}

fn print_normal_param_nocolor(param: &str, param_data: String) -> String {
    format!(
        "{}{} {}",
        style(param).bold(),
        style(":").bold(),
        param_data,
    )
}

fn print_raw_param(param: &str, param_data: &str) -> String {
    format!(
        "{}{}\n{}\n",
        style(param).bold(),
        style(":").bold(),
        style(param_data).blue()
    )
}

fn print_timestamp(timestamp: &OffsetDateTime) -> String {
    let formatted_timestamp = match timestamp.format(&Rfc3339) {
        Ok(timestamp) => style(timestamp).blue().to_string(),
        Err(_) => style("INVALID TIMESTAMP").red().to_string(),
    };

    print_normal_param_nocolor("timestamp", formatted_timestamp)
}
