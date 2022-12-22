use crate::client::http_utils::PollResponse;
use crate::crypto::rsa::RSAPrivKey;
use crate::interaction_log::LogEntry;

pub(super) trait ParseLogs {
    fn parse_logs(
        response: PollResponse,
        rsa_key: &RSAPrivKey,
    ) -> Result<Vec<LogEntry>, snafu::Whatever> {
        todo!()
    }
}
