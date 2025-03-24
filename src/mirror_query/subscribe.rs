// SPDX-License-Identifier: Apache-2.0

use async_stream::stream;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use futures_core::future::BoxFuture;
use futures_core::Stream;
use futures_util::StreamExt;
use tokio::time::sleep;
use tonic::transport::Channel;
use tonic::Status;

use crate::mirror_query::AnyMirrorQueryData;
use crate::{
    Client,
    Error,
    MirrorQuery,
};

impl<D> MirrorQuery<D>
where
    D: MirrorQueryExecute,
{
    /// Execute this query against the provided client of the Hedera network.
    // todo:
    #[allow(clippy::missing_errors_doc)]
    pub async fn execute(&mut self, client: &Client) -> crate::Result<D::Response> {
        self.execute_with_optional_timeout(client, None).await
    }

    pub(crate) async fn execute_with_optional_timeout(
        &self,
        client: &Client,
        timeout: Option<std::time::Duration>,
    ) -> crate::Result<D::Response> {
        self.data.execute_with_optional_timeout(&self.common, client, timeout).await
    }

    /// Execute this query against the provided client of the Hedera network.
    ///
    /// Note that `timeout` is the connection timeout.
    // todo:
    #[allow(clippy::missing_errors_doc)]
    pub async fn execute_with_timeout(
        &mut self,
        client: &Client,
        timeout: std::time::Duration,
    ) -> crate::Result<D::Response> {
        self.execute_with_optional_timeout(client, Some(timeout)).await
    }

    /// Subscribe to this query with the provided client of the Hedera network.
    pub fn subscribe<'a>(&self, client: &'a Client) -> D::ItemStream<'a> {
        self.subscribe_with_optional_timeout(client, None)
    }

    /// Subscribe to this query with the provided client of the Hedera network.
    ///
    /// Note that `timeout` is the connection timeout.
    pub fn subscribe_with_timeout<'a>(
        &self,
        client: &'a Client,
        timeout: std::time::Duration,
    ) -> D::ItemStream<'a> {
        self.subscribe_with_optional_timeout(client, Some(timeout))
    }

    pub(crate) fn subscribe_with_optional_timeout<'a>(
        &self,
        client: &'a Client,
        timeout: Option<std::time::Duration>,
    ) -> D::ItemStream<'a> {
        self.data.subscribe_with_optional_timeout(&self.common, client, timeout)
    }
}

pub trait MirrorQueryExecute: Sized + Into<AnyMirrorQueryData> + Send + Sync {
    type Item;
    type Response;
    type ItemStream<'a>: Stream<Item = crate::Result<Self::Item>> + 'a
    where
        Self: 'a;

    fn subscribe_with_optional_timeout<'a>(
        &self,
        params: &crate::mirror_query::MirrorQueryCommon,
        client: &'a crate::Client,
        timeout: Option<std::time::Duration>,
    ) -> Self::ItemStream<'a>
    where
        Self: 'a;

    fn execute_with_optional_timeout<'a>(
        &'a self,
        params: &'a super::MirrorQueryCommon,
        client: &'a Client,
        timeout: Option<std::time::Duration>,
    ) -> BoxFuture<'a, crate::Result<Self::Response>>;
}

impl<T> MirrorQueryExecute for T
where
    T: MirrorRequest + Sync + Clone + Into<AnyMirrorQueryData>,
{
    type Item = <Self as MirrorRequest>::Item;

    type Response = <Self as MirrorRequest>::Response;

    type ItemStream<'a>
        = <Self as MirrorRequest>::ItemStream<'a>
    where
        Self: 'a;

    fn subscribe_with_optional_timeout<'a>(
        &self,
        _params: &crate::mirror_query::MirrorQueryCommon,
        client: &'a crate::Client,
        timeout: Option<std::time::Duration>,
    ) -> Self::ItemStream<'a>
    where
        Self: 'a,
    {
        let timeout = timeout.or_else(|| client.request_timeout()).unwrap_or_else(|| {
            std::time::Duration::from_millis(backoff::default::MAX_ELAPSED_TIME_MILLIS)
        });

        // note: we don't care about keeping the mirrornet around, so, we just take the channel (which is arc-like)
        let channel = client.mirrornet().load().channel();

        Self::make_item_stream(crate::mirror_query::subscribe(channel, timeout, self.clone()))
    }

    fn execute_with_optional_timeout<'a>(
        &'a self,
        _params: &'a crate::mirror_query::MirrorQueryCommon,
        client: &crate::Client,
        timeout: Option<std::time::Duration>,
    ) -> BoxFuture<'a, crate::Result<Self::Response>> {
        let timeout = timeout.or_else(|| client.request_timeout()).unwrap_or_else(|| {
            std::time::Duration::from_millis(backoff::default::MAX_ELAPSED_TIME_MILLIS)
        });

        // note: we don't care about keeping the mirrornet around, so, we just take the channel (which is arc-like)
        let channel = client.mirrornet().load().channel();

        Self::try_collect(crate::mirror_query::subscribe(channel, timeout, self.clone()))
    }
}

pub trait MirrorRequest: Send {
    type GrpcItem: Send;
    type ConnectStream: Stream<Item = tonic::Result<Self::GrpcItem>> + Send;

    type Item;
    type Response;
    type Context: Default + Send + Sync;

    type ItemStream<'a>: Stream<Item = crate::Result<Self::Item>> + 'a;

    fn connect(
        &self,
        context: &Self::Context,
        channel: Channel,
    ) -> BoxFuture<'_, tonic::Result<Self::ConnectStream>>;

    /// Return `true` to retry establishing the stream, up to a configurable maximum timeout.
    #[allow(unused_variables)]
    fn should_retry(&self, status_code: tonic::Code) -> bool {
        false
    }

    fn make_item_stream<'a, S>(stream: S) -> Self::ItemStream<'a>
    where
        S: Stream<Item = crate::Result<Self::GrpcItem>> + Send + 'a;

    fn update_context(context: &mut Self::Context, item: &Self::GrpcItem);

    fn try_collect<'a, S>(stream: S) -> BoxFuture<'a, crate::Result<Self::Response>>
    where
        S: Stream<Item = crate::Result<Self::GrpcItem>> + Send + 'a;
}

pub(crate) fn subscribe<I: Send, R: MirrorRequest<GrpcItem = I> + Send + Sync>(
    channel: Channel,
    timeout: std::time::Duration,
    request: R,
) -> impl Stream<Item = crate::Result<I>> + Send {
    stream! {
        let request = request;

        let mut backoff = ExponentialBackoff {
            max_elapsed_time: Some(timeout),
            ..ExponentialBackoff::default()
        };

        let mut backoff_inf = ExponentialBackoff {
            max_elapsed_time: None,
            // remove maximum elapsed time for # of back-offs on inf.
            .. ExponentialBackoff::default()
        };

        let mut context = R::Context::default();

        loop {
            let status: Status = 'request: loop {
                // attempt to establish the stream
                let response = request.connect(&context, channel.clone()).await;

                let stream = match response {
                    // success, we now have a stream and may begin waiting for messages
                    Ok(stream) => stream,

                    Err(status) => {
                        break 'request status;
                    }
                };

                let mut stream = std::pin::pin!(stream);

                backoff.reset();
                backoff_inf.reset();

                #[allow(unused_labels)]
                'message: loop {
                    let message = stream.next().await.transpose();

                    let message = match message {
                        Ok(Some(message)) => message,
                        Ok(None) => {
                            // end of stream
                            // hopefully due to configured limits or expected conditions
                            return;
                        }

                        Err(status) => {
                            break 'request status;
                        }
                    };

                    R::update_context(&mut context, &message);

                    yield Ok(message);
                }
            };

            match status.code() {
                tonic::Code::Unavailable | tonic::Code::ResourceExhausted => {
                    // encountered a temporarily down or overloaded service
                    sleep(backoff_inf.next_backoff().unwrap()).await;
                }

                tonic::Code::Unknown if status.message() == "error reading a body from connection: connection reset" => {
                    // connection was aborted by the server
                    sleep(backoff_inf.next_backoff().unwrap()).await;
                }

                code if request.should_retry(code) => {
                    if let Some(duration) = backoff.next_backoff() {
                        sleep(duration).await;
                    } else {
                        // maximum time allowed has elapsed
                        // NOTE: it should be impossible to reach here without capturing at least one error
                        yield Err(Error::TimedOut(Error::from(status).into()));
                        return;
                    }
                }
                _ => {
                    // encountered an un-recoverable failure when attempting
                    // to establish the stream
                    yield Err(Error::from(status));
                    return;
                }
            }
        }
    }
}
