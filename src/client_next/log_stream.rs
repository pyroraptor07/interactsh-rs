use std::sync::Arc;
use std::time::Duration;

use futures::future::BoxFuture;
use futures::Stream;
use parking_lot::RwLock;

use super::log_parsing::ParseLogs;
use super::{ClientStatus, CommInfo, LogPollResult};
use crate::crypto::rsa::RSAPrivKey;


enum LogStreamStatus<'status> {
    Ready,
    ErrorReturned,
    Closed,
    WaitingOnServer(BoxFuture<'status, Result<reqwest::Response, reqwest::Error>>),
    WaitingOnTimer(BoxFuture<'status, ()>),
}

pub(super) struct LogStream<'a> {
    client_status: Arc<RwLock<ClientStatus>>,
    rsa_key: Arc<RSAPrivKey>,
    server_comm_info: Arc<CommInfo>,
    reqwest_client: Arc<reqwest::Client>,
    parse_logs: bool,
    poll_period: Duration,
    stream_status: LogStreamStatus<'a>,
}

impl<'a> LogStream<'a> {
    pub fn new(
        client_status: Arc<RwLock<ClientStatus>>,
        rsa_key: Arc<RSAPrivKey>,
        server_comm_info: Arc<CommInfo>,
        reqwest_client: Arc<reqwest::Client>,
        parse_logs: bool,
        poll_period: Duration,
    ) -> LogStream<'a> {
        let stream_status = LogStreamStatus::Ready;

        Self {
            client_status,
            rsa_key,
            server_comm_info,
            reqwest_client,
            parse_logs,
            poll_period,
            stream_status,
        }
    }
}

impl Stream for LogStream<'_> {
    type Item = LogPollResult;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}

impl ParseLogs for LogStream<'_> {}
