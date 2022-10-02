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

/// A fully parsed log entry returned by an Interactsh server
#[derive(Debug, Deserialize)]
#[serde(tag = "protocol")]
pub enum ParsedLogEntry {
    #[serde(alias = "dns", rename_all(deserialize = "kebab-case"))]
    Dns {
        unique_id: String,
        full_id: String,
        q_type: DnsQType,
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
    use time::OffsetDateTime;
    use time::format_description::well_known::Iso8601;

    // #[allow(unused)]
    // pub fn serialize<S: Serializer>(
    //     datetime: &OffsetDateTime,
    //     serializer: S,
    // ) -> Result<S::Ok, S::Error> {
    //     let serialized_string = datetime.format(&Iso8601::DEFAULT)
    //         .map_err(|e| ser::Error::custom(format!("{}", e)))?;
    //     serializer.serialize_str(&serialized_string)
    // }

    pub fn deserialize<'a, D: Deserializer<'a>>(deserializer: D) -> Result<OffsetDateTime, D::Error> {
        OffsetDateTime::parse(
            <_>::deserialize(deserializer)?, 
            &Iso8601::DEFAULT,
        ).map_err(|e| de::Error::custom(format!("{}", e)))
    }
}
