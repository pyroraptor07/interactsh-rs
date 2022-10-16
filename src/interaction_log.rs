use std::fmt::Display;

use serde::Deserialize;
use time::OffsetDateTime;


/// Type returned when a [RegisteredClient](crate::client::RegisteredClient)
/// polls a server and obtains new interaction logs
///
/// Whether or not a raw log or a parsed log is
/// returned depends on the following:
/// 1. If the client was built with the "parse logs" option set to true
/// (see [ClientBuilder](crate::client::ClientBuilder))
/// 2. If the logs are able to be parsed (if the logs are unable to be parsed, then the raw
/// logs are returned)
#[derive(Debug)]
pub enum LogEntry {
    ParsedLog(ParsedLogEntry),
    RawLog(RawLog),
}

impl LogEntry {
    #[allow(dead_code)]
    pub(crate) fn return_raw_log(raw_log_str: &str) -> LogEntry {
        let raw_log = RawLog {
            log_entry: raw_log_str.to_owned(),
        };

        Self::RawLog(raw_log)
    }

    #[allow(dead_code)]
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


mod timestamp_unixstr_parse {
    use serde::{de, Deserialize, Deserializer};
    use time::format_description::well_known::Rfc3339;
    use time::OffsetDateTime;

    pub fn deserialize<'a, D: Deserializer<'a>>(
        deserializer: D,
    ) -> Result<OffsetDateTime, D::Error> {
        OffsetDateTime::parse(<_>::deserialize(deserializer)?, &Rfc3339)
            .map_err(|e| de::Error::custom(format!("{}", e)))
    }
}

#[cfg(test)]
mod tests {
    use fake::{faker, Fake};
    use rand::distributions::Slice;
    use rand::Rng;
    use serde_json::{json, Value};
    use svix_ksuid::*;
    use time::format_description::well_known::Rfc3339;
    use time::OffsetDateTime;

    use super::*;

    fn get_random_id() -> String {
        let ksuid_a = Ksuid::new(None, None).to_string().to_ascii_lowercase();
        let ksuid_b = Ksuid::new(None, None).to_string().to_ascii_lowercase();
        let mut random_id = format!("{}{}", ksuid_a, ksuid_b);
        random_id.truncate(33);

        random_id
    }

    fn get_timestamp() -> String {
        OffsetDateTime::now_utc().format(&Rfc3339).unwrap()
    }

    fn get_ip_address() -> String {
        faker::internet::en::IP().fake()
    }

    fn get_paragraph() -> String {
        faker::lorem::en::Paragraph(1..2).fake()
    }

    fn get_email_address() -> String {
        faker::internet::en::SafeEmail().fake()
    }

    fn get_random_dns_q_type() -> String {
        let mut rng = rand::thread_rng();
        let q_types = ["A", "NS", "CNAME", "SOA", "PTR", "MX", "TXT", "AAAA"];
        let q_types_dist = Slice::new(&q_types).unwrap();

        rng.sample(q_types_dist).to_string()
    }

    fn try_parse_json(json_value: Value) -> LogEntry {
        let json_value_string =
            serde_json::to_string(&json_value).expect("Unable to parse json to string");
        LogEntry::try_parse_log(&json_value_string)
    }

    fn get_raw_log(json_value: Value) -> LogEntry {
        let json_value_string =
            serde_json::to_string(&json_value).expect("Unable to parse json to string");
        LogEntry::return_raw_log(&json_value_string)
    }

    #[test]
    fn log_entry_successfully_parses_valid_dns_log_no_qtype() {
        let random_id = get_random_id();
        let timestamp = get_timestamp();
        let remote_address = get_ip_address();
        let raw_request = get_paragraph();
        let raw_response = get_paragraph();

        let json_log = json!({
            "protocol": "dns",
            "unique-id": random_id,
            "full-id": random_id,
            "raw-request": raw_request,
            "raw-response": raw_response,
            "remote-address": remote_address,
            "timestamp": timestamp
        });

        let log_parse_result = try_parse_json(json_log);

        match log_parse_result {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Dns { .. } => {}
                    _ => panic!("DNS log did not parse to DNS variant"),
                }
            }
            LogEntry::RawLog(_) => panic!("DNS log did not parse at all"),
        }
    }

    #[test]
    fn log_entry_successfully_parses_valid_dns_log_with_qtype() {
        let random_id = get_random_id();
        let timestamp = get_timestamp();
        let remote_address = get_ip_address();
        let raw_request = get_paragraph();
        let raw_response = get_paragraph();
        let q_type = get_random_dns_q_type();

        let json_log = json!({
            "protocol": "dns",
            "unique-id": random_id,
            "full-id": random_id,
            "q-type": q_type,
            "raw-request": raw_request,
            "raw-response": raw_response,
            "remote-address": remote_address,
            "timestamp": timestamp
        });

        let log_parse_result = try_parse_json(json_log);

        match log_parse_result {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Dns { .. } => {}
                    _ => panic!("DNS log did not parse to DNS variant"),
                }
            }
            LogEntry::RawLog(_) => panic!("DNS log did not parse at all"),
        }
    }

    #[test]
    fn log_entry_successfully_parses_valid_http_log() {
        let random_id = get_random_id();
        let timestamp = get_timestamp();
        let remote_address = get_ip_address();
        let raw_request = get_paragraph();
        let raw_response = get_paragraph();

        let json_log = json!({
            "protocol": "http",
            "unique-id": random_id,
            "full-id": random_id,
            "raw-request": raw_request,
            "raw-response": raw_response,
            "remote-address": remote_address,
            "timestamp": timestamp
        });

        let log_parse_result = try_parse_json(json_log);

        match log_parse_result {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Http { .. } => {}
                    _ => panic!("HTTP log did not parse to HTTP variant"),
                }
            }
            LogEntry::RawLog(_) => panic!("HTTP log did not parse at all"),
        }
    }

    #[test]
    fn log_entry_successfully_parses_valid_ftp_log() {
        let timestamp = get_timestamp();
        let remote_address = get_ip_address();
        let raw_request = get_paragraph();

        let json_log = json!({
            "protocol": "ftp",
            "raw-request": raw_request,
            "remote-address": remote_address,
            "timestamp": timestamp
        });

        let log_parse_result = try_parse_json(json_log);

        match log_parse_result {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Ftp { .. } => {}
                    _ => panic!("FTP log did not parse to FTP variant"),
                }
            }
            LogEntry::RawLog(_) => panic!("FTP log did not parse at all"),
        }
    }

    #[test]
    fn log_entry_successfully_parses_valid_ldap_log() {
        let random_id = get_random_id();
        let timestamp = get_timestamp();
        let remote_address = get_ip_address();
        let raw_request = get_paragraph();
        let raw_response = get_paragraph();

        let json_log = json!({
            "protocol": "ldap",
            "unique-id": random_id,
            "full-id": random_id,
            "raw-request": raw_request,
            "raw-response": raw_response,
            "remote-address": remote_address,
            "timestamp": timestamp
        });

        let log_parse_result = try_parse_json(json_log);

        match log_parse_result {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Ldap { .. } => {}
                    _ => panic!("LDAP log did not parse to LDAP variant"),
                }
            }
            LogEntry::RawLog(_) => panic!("LDAP log did not parse at all"),
        }
    }

    #[test]
    fn log_entry_successfully_parses_valid_smb_log() {
        let timestamp = get_timestamp();
        let raw_request = get_paragraph();

        let json_log = json!({
            "protocol": "smb",
            "raw-request": raw_request,
            "timestamp": timestamp
        });

        let log_parse_result = try_parse_json(json_log);

        match log_parse_result {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Smb { .. } => {}
                    _ => panic!("SMB log did not parse to SMB variant"),
                }
            }
            LogEntry::RawLog(_) => panic!("SMB log did not parse at all"),
        }
    }

    #[test]
    fn log_entry_successfully_parses_valid_smtp_log() {
        let random_id = get_random_id();
        let timestamp = get_timestamp();
        let remote_address = get_ip_address();
        let raw_request = get_paragraph();
        let email_address = get_email_address();

        let json_log = json!({
            "protocol": "smtp",
            "unique-id": random_id,
            "full-id": random_id,
            "raw-request": raw_request,
            "smtp-from": email_address,
            "remote-address": remote_address,
            "timestamp": timestamp
        });

        let log_parse_result = try_parse_json(json_log);

        match log_parse_result {
            LogEntry::ParsedLog(parsed_log) => {
                match parsed_log {
                    ParsedLogEntry::Smtp { .. } => {}
                    _ => panic!("SMTP log did not parse to SMTP variant"),
                }
            }
            LogEntry::RawLog(_) => panic!("SMTP log did not parse at all"),
        }
    }

    #[test]
    fn log_entry_returns_raw_log_for_invalid_log() {
        let random_id = get_random_id();
        let timestamp = get_timestamp();
        let remote_address: String = get_ip_address();
        let raw_request: String = get_paragraph();

        let json_log = json!({
            "protocol": "http",
            "unique-id": random_id,
            "full-id": random_id,
            "raw-request": raw_request,
            "remote-address": remote_address,
            "timestamp": timestamp,
            "unexpected-field": "unexpected field"
        });

        let log_parse_result = try_parse_json(json_log);

        match log_parse_result {
            LogEntry::ParsedLog(_) => panic!("Expected raw log, got a parsed log"),
            LogEntry::RawLog(_) => {}
        }
    }

    #[test]
    fn log_entry_successfully_returns_raw_log() {
        let random_id = get_random_id();
        let timestamp = get_timestamp();
        let remote_address: String = get_ip_address();
        let raw_request: String = get_paragraph();
        let raw_response: String = get_paragraph();

        let json_log = json!({
            "protocol": "http",
            "unique-id": random_id,
            "full-id": random_id,
            "raw-request": raw_request,
            "raw-response": raw_response,
            "remote-address": remote_address,
            "timestamp": timestamp
        });

        let log_entry = get_raw_log(json_log);

        match log_entry {
            LogEntry::ParsedLog(_) => panic!("Expected raw log, got a parsed log"),
            LogEntry::RawLog(_) => {}
        }
    }
}
