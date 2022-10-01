use serde::{Serialize, Deserialize};
use time::OffsetDateTime;

/// Wrapper struct around a log entry.
/// 
/// Returned when [Client] polls the server and receives
/// a new log.
#[derive(Debug, serde::Deserialize)]
pub struct LogEntry {
    pub protocol: String,
    
    #[serde(rename(deserialize = "unique-id"))]
    pub unique_id: String,

    #[serde(rename(deserialize = "full-id"))]
    pub full_id: String,

    #[serde(rename(deserialize = "q-type"))]
    pub q_type: Option<String>,

    #[serde(rename(deserialize = "raw-request"))]
    pub raw_request: String,

    #[serde(rename(deserialize = "raw-response"))]
    pub raw_response: String,

    #[serde(rename(deserialize = "remote-address"))]
    pub remote_address: std::net::IpAddr,

    #[serde(with = "timestamp_unixstr_parse")]
    pub timestamp: OffsetDateTime,
}

mod timestamp_unixstr_parse {

    use serde::{de, Deserialize, Deserializer, ser, Serializer};
    use time::OffsetDateTime;
    use time::format_description::well_known::Iso8601;

    #[allow(unused)]
    pub fn serialize<S: Serializer>(
        datetime: &OffsetDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let serialized_string = datetime.format(&Iso8601::DEFAULT)
            .map_err(|e| ser::Error::custom(format!("{}", e)))?;
        serializer.serialize_str(&serialized_string)
    }

    pub fn deserialize<'a, D: Deserializer<'a>>(deserializer: D) -> Result<OffsetDateTime, D::Error> {
        OffsetDateTime::parse(
            <_>::deserialize(deserializer)?, 
            &Iso8601::DEFAULT,
        ).map_err(|e| de::Error::custom(format!("{}", e)))
    }
}

#[derive(Debug)]
pub struct RawLog {
    pub log_entry: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "protocol")]
pub enum ParsedLogEntry {
    #[serde(alias = "dns")]
    Dns,

    #[serde(alias = "ftp")]
    Ftp,

    #[serde(alias = "http")]
    Http,

    #[serde(alias = "ldap")]
    Ldap,

    #[serde(alias = "smb")]
    Smb,
    
    #[serde(alias = "smtp")]
    Smtp,

    #[serde(skip)]
    Other {
        raw_log_entry: String,
    },
}
