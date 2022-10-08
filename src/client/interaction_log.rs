use std::fmt::Display;

use serde::Deserialize;
use time::OffsetDateTime;


/// Type returned when a [RegisteredClient](crate::client::registered::RegisteredClient)
/// polls a server and obtains new interaction logs
///
/// Whether or not a raw log or a parsed log is
/// returned depends on the following:
/// 1. If the client was built with the "parse logs" option set to true
/// (see [ClientBuilder](crate::client::builder::ClientBuilder))
/// 2. If the logs are able to be parsed (if the logs are unable to be parsed, then the raw
/// logs are returned)
#[derive(Debug)]
pub enum LogEntry {
    ParsedLog(ParsedLogEntry),
    RawLog(RawLog),
}

impl LogEntry {
    pub(crate) fn return_raw_log(raw_log_str: &str) -> LogEntry {
        let raw_log = RawLog {
            log_entry: raw_log_str.to_owned(),
        };

        Self::RawLog(raw_log)
    }

    pub(crate) fn try_parse_log(raw_log_str: &str) -> LogEntry {
        match serde_json::from_str::<ParsedLogEntry>(raw_log_str) {
            Ok(parsed_log) => Self::ParsedLog(parsed_log),
            Err(_) => Self::return_raw_log(raw_log_str),
        }
    }
}

/// Wrapper type containing the raw log string received by the client from the
/// Interactsh server (after decoding and decrypting)
#[derive(Debug)]
pub struct RawLog {
    pub log_entry: String,
}

#[derive(Debug, Deserialize)]
pub enum DnsQType {
    A,
    NS,
    CNAME,
    SOA,
    PTR,
    MX,
    TXT,
    AAAA,
}

impl Display for DnsQType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsQType::A => write!(f, "A"),
            DnsQType::NS => write!(f, "NS"),
            DnsQType::CNAME => write!(f, "CNAME"),
            DnsQType::SOA => write!(f, "SOA"),
            DnsQType::PTR => write!(f, "PTR"),
            DnsQType::MX => write!(f, "MX"),
            DnsQType::TXT => write!(f, "TXT"),
            DnsQType::AAAA => write!(f, "AAAA"),
        }
    }
}

/// A fully parsed log entry returned by an Interactsh server
#[derive(Debug, Deserialize)]
#[serde(tag = "protocol")]
pub enum ParsedLogEntry {
    #[serde(alias = "dns", rename_all(deserialize = "kebab-case"))]
    Dns {
        unique_id: String,
        full_id: String,
        q_type: Option<DnsQType>,
        raw_request: String,
        raw_response: String,
        remote_address: std::net::IpAddr,
        #[serde(with = "timestamp_unixstr_parse")]
        timestamp: OffsetDateTime,
    },

    #[serde(alias = "ftp", rename_all(deserialize = "kebab-case"))]
    Ftp {
        remote_address: std::net::IpAddr,
        raw_request: String,
        #[serde(with = "timestamp_unixstr_parse")]
        timestamp: OffsetDateTime,
    },

    #[serde(alias = "http", rename_all(deserialize = "kebab-case"))]
    Http {
        unique_id: String,
        full_id: String,
        raw_request: String,
        raw_response: String,
        remote_address: std::net::IpAddr,
        #[serde(with = "timestamp_unixstr_parse")]
        timestamp: OffsetDateTime,
    },

    #[serde(alias = "ldap", rename_all(deserialize = "kebab-case"))]
    Ldap {
        unique_id: String,
        full_id: String,
        raw_request: String,
        raw_response: String,
        remote_address: std::net::IpAddr,
        #[serde(with = "timestamp_unixstr_parse")]
        timestamp: OffsetDateTime,
    },

    #[serde(alias = "smb", rename_all(deserialize = "kebab-case"))]
    Smb {
        raw_request: String,
        #[serde(with = "timestamp_unixstr_parse")]
        timestamp: OffsetDateTime,
    },

    #[serde(alias = "smtp", rename_all(deserialize = "kebab-case"))]
    Smtp {
        unique_id: String,
        full_id: String,
        raw_request: String,
        smtp_from: String,
        remote_address: std::net::IpAddr,
        #[serde(with = "timestamp_unixstr_parse")]
        timestamp: OffsetDateTime,
    },
}

impl Display for ParsedLogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let log_string = match self {
            ParsedLogEntry::Dns {
                unique_id,
                full_id,
                q_type,
                raw_request,
                raw_response,
                remote_address,
                timestamp,
            } => {
                format!(
                    "[DNS]\nUnique ID: {}\nFull ID: {}\nQ Type: {}\nRaw Request:\n{}\nRaw Response:\n{}\nRemote Address: {}\nTimestamp: {}",
                    unique_id,
                    full_id,
                    if let Some(inner) = q_type {inner.to_string()} else {"None".to_owned()},
                    raw_request,
                    raw_response,
                    remote_address.to_string(),
                    timestamp.to_string(),
                )
            }
            ParsedLogEntry::Ftp {
                remote_address,
                raw_request,
                timestamp,
            } => {
                format!(
                    "[FTP]\nRemote Address: {}\nRaw Request:\n{}\nTimestamp: {}",
                    remote_address.to_string(),
                    raw_request,
                    timestamp.to_string(),
                )
            }
            ParsedLogEntry::Http {
                unique_id,
                full_id,
                raw_request,
                raw_response,
                remote_address,
                timestamp,
            } => {
                format!(
                    "[HTTP]\nUnique ID: {}\nFull ID: {}\nRaw Request:\n{}\nRaw Response:\n{}\nRemote Address: {}\nTimestamp: {}",
                    unique_id,
                    full_id,
                    raw_request,
                    raw_response,
                    remote_address.to_string(),
                    timestamp.to_string(),
                )
            }
            ParsedLogEntry::Ldap {
                unique_id,
                full_id,
                raw_request,
                raw_response,
                remote_address,
                timestamp,
            } => {
                format!(
                    "[LDAP]\nUnique ID: {}\nFull ID: {}\nRaw Request:\n{}\nRaw Response:\n{}\nRemote Address: {}\nTimestamp: {}",
                    unique_id,
                    full_id,
                    raw_request,
                    raw_response,
                    remote_address.to_string(),
                    timestamp.to_string(),
                )
            }
            ParsedLogEntry::Smb {
                raw_request,
                timestamp,
            } => {
                format!(
                    "[SMB]\nRaw Request:\n{}\nTimestamp: {}",
                    raw_request,
                    timestamp.to_string(),
                )
            }
            ParsedLogEntry::Smtp {
                unique_id,
                full_id,
                raw_request,
                smtp_from,
                remote_address,
                timestamp,
            } => {
                format!(
                    "[SMTP]\nUnique ID: {}\nFull ID: {}\nRaw Request:\n{}\nSMTP From: {}\nRemote Address: {}\nTimestamp: {}",
                    unique_id,
                    full_id,
                    raw_request,
                    smtp_from,
                    remote_address.to_string(),
                    timestamp.to_string(),
                )
            }
        };

        write!(f, "{}", log_string)
    }
}


mod timestamp_unixstr_parse {
    use serde::{de, Deserialize, Deserializer};
    use time::format_description::well_known::Iso8601;
    use time::OffsetDateTime;

    pub fn deserialize<'a, D: Deserializer<'a>>(
        deserializer: D,
    ) -> Result<OffsetDateTime, D::Error> {
        OffsetDateTime::parse(<_>::deserialize(deserializer)?, &Iso8601::DEFAULT)
            .map_err(|e| de::Error::custom(format!("{}", e)))
    }
}
