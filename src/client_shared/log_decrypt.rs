use base64::engine::general_purpose;
use base64::Engine as _;
use snafu::ResultExt;

use super::http_utils::PollResponse;
use crate::client_shared::errors::*;
use crate::crypto::aes;
use crate::crypto::rsa::RSAPrivKey;
use crate::interaction_log::LogEntry;

pub(crate) fn decrypt_logs(
    response: PollResponse,
    rsa_key: &RSAPrivKey,
    parse_logs: bool,
) -> Result<Option<Vec<LogEntry>>, PollError> {
    // Return None if empty
    let response_body_data = match response.data_list {
        Some(data) => {
            if data.is_empty() {
                return Ok(None);
            } else {
                data
            }
        }
        None => return Ok(None),
    };

    // Decode and decrypt AES key
    let aes_key_decoded = general_purpose::STANDARD
        .decode(&response.aes_key)
        .context(poll_error::AesBase64DecodeFailedSnafu)?;
    let aes_plain_key = rsa_key
        .decrypt_data(&aes_key_decoded)
        .context(poll_error::AesKeyDecryptFailedSnafu)?;

    // Decode and decrypt logs
    let mut decrypted_logs = Vec::new();
    for encoded_data in response_body_data.iter() {
        let encrypted_data = general_purpose::STANDARD
            .decode(encoded_data)
            .context(poll_error::DataBase64DecodeFailedSnafu)?;

        let decrypted_data = aes::decrypt_data(&aes_plain_key, &encrypted_data)
            .context(poll_error::DataDecryptFailedSnafu)?;

        let decrypted_string = String::from_utf8_lossy(&decrypted_data);

        let log_entry = LogEntry::new_log_entry(&decrypted_string, parse_logs);

        decrypted_logs.push(log_entry);
    }

    Ok(Some(decrypted_logs))
}
