use crate::client::http_utils::PollResponse;
use crate::crypto::rsa::RSAPrivKey;

pub(super) trait ParseLogs {
    fn parse_logs(response: PollResponse, rsa_key: &RSAPrivKey) -> Result<String, snafu::Whatever> {
        todo!()
    }
}
