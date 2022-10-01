use serde::Deserialize;
use time::OffsetDateTime;


pub fn try_parse_log(data_str: &str) -> LogEntry {
    match serde_json::from_str::<ParsedLogEntry>(data_str) {
        Ok(parsed_log) => LogEntry::ParsedLog(parsed_log),
        Err(_) => {
            let raw_log = RawLog {
                log_entry: data_str.to_owned(),
            };
            LogEntry::RawLog(raw_log)
        }
    }
}

#[derive(Debug)]
pub enum LogEntry {
    ParsedLog(ParsedLogEntry),
    RawLog(RawLog),
}

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

    #[serde(skip)]
    Other {
        raw_log_entry: String,
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
