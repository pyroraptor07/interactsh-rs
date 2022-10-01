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
    pub remote_address: String,

    pub timestamp: String,
}