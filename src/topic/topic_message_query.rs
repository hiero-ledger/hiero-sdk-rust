// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::{
    mem,
    task,
};

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::TryStreamExt;
use hedera_proto::mirror;
use hedera_proto::mirror::consensus_service_client::ConsensusServiceClient;
use hedera_proto::mirror::ConsensusTopicQuery;
use time::{
    Duration,
    OffsetDateTime,
};
use tonic::transport::Channel;
use tonic::Response;

use super::topic_message::{
    PbTopicMessageChunk,
    PbTopicMessageHeader,
};
use crate::mirror_query::{
    AnyMirrorQueryData,
    AnyMirrorQueryMessage,
    MirrorRequest,
};
use crate::protobuf::FromProtobuf;
use crate::{
    AnyMirrorQueryResponse,
    MirrorQuery,
    ToProtobuf,
    TopicId,
    TopicMessage,
    TransactionId,
};

// TODO: test, test, and test
// TODO: investigate failure scenarios

// TODO: validate checksums after PR is merged

#[derive(Default)]
pub struct TopicMessageQueryContext {
    start_time: Option<OffsetDateTime>,
}

/// Query a stream of Hiero Consensus Service (HCS)
/// messages for an HCS Topic via a specific (possibly open-ended) time range.
pub type TopicMessageQuery = MirrorQuery<TopicMessageQueryData>;

#[derive(Debug, Default, Clone)]
pub struct TopicMessageQueryData {
    /// The topic ID to retrieve messages for.
    topic_id: Option<TopicId>,

    /// Include messages which reached consensus on or after this time.
    /// Defaults to the current time.
    start_time: Option<OffsetDateTime>,

    /// Include messages which reached consensus before this time.
    end_time: Option<OffsetDateTime>,

    /// The maximum number of messages to receive before stopping.
    limit: u64,
}

impl TopicMessageQueryData {
    fn map_stream<'a, S>(stream: S) -> impl Stream<Item = crate::Result<TopicMessage>>
    where
        S: Stream<Item = crate::Result<mirror::ConsensusTopicResponse>> + Send + 'a,
    {
        MessagesMapStream { inner: stream, incomplete_messages: HashMap::new() }
    }
}

impl TopicMessageQuery {
    /// Returns the ID of the topic to retrieve messages for.
    #[must_use]
    pub fn get_topic_id(&self) -> Option<TopicId> {
        self.data.topic_id
    }

    /// Sets the topic ID to retrieve messages for.
    pub fn topic_id(&mut self, id: impl Into<TopicId>) -> &mut Self {
        self.data.topic_id = Some(id.into());
        self
    }

    /// Returns the minimum `consensus_timestamp` of the messages to return.
    #[must_use]
    pub fn get_start_time(&self) -> Option<OffsetDateTime> {
        self.data.start_time
    }

    /// Sets to include messages which reached consensus on or after this time.
    /// Defaults to the current time.
    pub fn start_time(&mut self, time: OffsetDateTime) -> &mut Self {
        self.data.start_time = Some(time);
        self
    }

    /// Returns the maximum `consensus_timestamp` of the messages to return.
    #[must_use]
    pub fn get_end_time(&self) -> Option<OffsetDateTime> {
        self.data.end_time
    }

    /// Sets to include messages which reached consensus before this time.
    pub fn end_time(&mut self, time: OffsetDateTime) -> &mut Self {
        self.data.end_time = Some(time);
        self
    }

    /// Returns maximum number of messages to be returned.
    #[must_use]
    pub fn get_limit(&self) -> u64 {
        self.data.limit
    }

    /// Sets the maximum number of messages to be returned, before closing the subscription.
    /// Defaults to _unlimited_.
    pub fn limit(&mut self, limit: u64) -> &mut Self {
        self.data.limit = limit;
        self
    }
}

impl From<TopicMessageQueryData> for AnyMirrorQueryData {
    fn from(data: TopicMessageQueryData) -> Self {
        Self::TopicMessage(data)
    }
}

impl MirrorRequest for TopicMessageQueryData {
    type GrpcItem = mirror::ConsensusTopicResponse;

    type ConnectStream = tonic::Streaming<Self::GrpcItem>;

    type Context = TopicMessageQueryContext;

    type Item = TopicMessage;

    type Response = Vec<TopicMessage>;

    type ItemStream<'a> = BoxStream<'a, crate::Result<TopicMessage>>;

    fn connect(
        &self,
        context: &Self::Context,
        channel: Channel,
    ) -> BoxFuture<'_, tonic::Result<Self::ConnectStream>> {
        let topic_id = self.topic_id.to_protobuf();

        let consensus_end_time = self.end_time.map(Into::into);

        // If we had to reconnect, we want to start 1ns after the last message we recieved.
        // We don't want to start *at* the last message we recieved because that'd give us that message again.
        let consensus_start_time = context
            .start_time
            .map(|it| it.checked_add(Duration::nanoseconds(1)).unwrap())
            .or(self.start_time)
            .map(Into::into);

        let request = ConsensusTopicQuery {
            consensus_end_time,
            consensus_start_time,
            topic_id,
            limit: self.limit,
        };

        Box::pin(async move {
            ConsensusServiceClient::new(channel)
                .subscribe_topic(request)
                .await
                .map(Response::into_inner)
        })
    }

    fn make_item_stream<'a, S>(stream: S) -> Self::ItemStream<'a>
    where
        S: Stream<Item = crate::Result<Self::GrpcItem>> + Send + 'a,
    {
        Box::pin(Self::map_stream(stream))
    }

    fn try_collect<'a, S>(stream: S) -> BoxFuture<'a, crate::Result<Self::Response>>
    where
        S: Stream<Item = crate::Result<Self::GrpcItem>> + Send + 'a,
    {
        // this doesn't reuse the work in `make_item_stream`
        Box::pin(Self::map_stream(stream).try_collect())
    }

    fn update_context(context: &mut Self::Context, item: &Self::GrpcItem) {
        context.start_time =
            item.consensus_timestamp.map(OffsetDateTime::from).or(context.start_time);
    }
}

impl From<TopicMessage> for AnyMirrorQueryMessage {
    fn from(value: TopicMessage) -> Self {
        Self::TopicMessage(value)
    }
}

impl From<Vec<TopicMessage>> for AnyMirrorQueryResponse {
    fn from(value: Vec<TopicMessage>) -> Self {
        Self::TopicMessage(value)
    }
}

enum IncompleteMessage {
    Partial(OffsetDateTime, Vec<PbTopicMessageChunk>),
    Expired,
    Complete,
}

impl IncompleteMessage {
    fn handle_expiry(&mut self) -> &mut Self {
        match self {
            IncompleteMessage::Partial(expiry, _) if *expiry < OffsetDateTime::now_utc() => {
                *self = Self::Expired;
            }
            _ => {}
        }

        self
    }
}

pin_project_lite::pin_project! {
    struct MessagesMapStream<S> {
        #[pin]
        inner: S,
        incomplete_messages: HashMap<TransactionId, IncompleteMessage>,
    }
}

impl<S> Stream for MessagesMapStream<S>
where
    S: Stream<Item = crate::Result<mirror::ConsensusTopicResponse>> + Send,
{
    type Item = crate::Result<TopicMessage>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        use task::Poll;

        let mut this = self.project();

        loop {
            let item = match task::ready!(this.inner.as_mut().poll_next(cx)) {
                Some(Ok(item)) => item,
                Some(Err(e)) => return Poll::Ready(Some(Err(e))),
                None => return Poll::Ready(None),
            };

            match filter_map(item, this.incomplete_messages) {
                Ok(Some(item)) => return Poll::Ready(Some(Ok(item))),
                Ok(None) => {}
                Err(e) => return Poll::Ready(Some(Err(e))),
            }
        }
    }
}

fn filter_map(
    mut item: mirror::ConsensusTopicResponse,
    incomplete_messages: &mut HashMap<TransactionId, IncompleteMessage>,
) -> crate::Result<Option<TopicMessage>> {
    let header = PbTopicMessageHeader {
        consensus_timestamp: pb_getf!(item, consensus_timestamp)?.into(),
        sequence_number: item.sequence_number,
        running_hash: item.running_hash,
        running_hash_version: item.running_hash_version,
        message: item.message,
    };

    let item = match item.chunk_info.take() {
        Some(chunk_info) if chunk_info.total > 1 => PbTopicMessageChunk {
            header,
            initial_transaction_id: TransactionId::from_protobuf(pb_getf!(
                chunk_info,
                initial_transaction_id
            )?)?,
            number: chunk_info.number,
            total: chunk_info.total,
        },
        _ => return Ok(Some(TopicMessage::from_single(header))),
    };

    let tx_id = item.initial_transaction_id;

    let entry = incomplete_messages.entry(tx_id).or_insert_with(|| {
        IncompleteMessage::Partial(
            // todo: configurable?
            OffsetDateTime::now_utc() + time::Duration::minutes(15),
            Vec::new(),
        )
    });

    let IncompleteMessage::Partial(_, messages) = entry.handle_expiry() else { return Ok(None) };

    match messages.binary_search_by_key(&item.number, |it| it.number) {
        // We have a duplicate `number`, so, we'll just ignore it (this is unspecified behavior)
        Ok(_) => {}
        Err(index) => messages.insert(index, item),
    };

    // find the smallest `total` so that we aren't susceptable to stuff like total changing (and getting bigger)
    // later on there's a check that ensures that they all have the same total.
    let total = messages.iter().map(|it| it.total).min().unwrap();

    // note: because of the way we handle `total`, `total` can get *smaller*.

    match messages.len() >= total as usize {
        true => {
            let messages = mem::take(messages);
            *entry = IncompleteMessage::Complete;
            Ok(Some(TopicMessage::from_chunks(messages)))
        }

        false => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use crate::{
        TopicId,
        TopicMessageQuery,
    };

    #[test]
    fn get_set_topic_id() {
        let mut query = TopicMessageQuery::new();
        query.topic_id(TopicId::new(31, 41, 59));

        assert_eq!(query.get_topic_id(), Some(TopicId::new(31, 41, 59)));
    }
    #[test]
    fn get_set_start_time() {
        let start_time = OffsetDateTime::now_utc();

        let mut query = TopicMessageQuery::new();
        query.start_time(start_time);

        assert_eq!(query.get_start_time(), Some(start_time));
    }
    #[test]
    fn get_set_end_time() {
        let end_time = OffsetDateTime::now_utc();

        let mut query = TopicMessageQuery::new();
        query.end_time(end_time);

        assert_eq!(query.get_end_time(), Some(end_time));
    }
    #[test]
    fn get_set_limit() {
        let mut query = TopicMessageQuery::new();
        query.limit(1415);

        assert_eq!(query.get_limit(), 1415);
    }
}
