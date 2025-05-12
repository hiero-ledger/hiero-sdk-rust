// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::consensus_service_client::ConsensusServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    BoxGrpcFuture,
    Error,
    Query,
    ToProtobuf,
    TopicId,
    TopicInfo,
    ValidateChecksums,
};

/// Retrieve the latest state of a topic.
pub type TopicInfoQuery = Query<TopicInfoQueryData>;

#[derive(Default, Clone, Debug)]
pub struct TopicInfoQueryData {
    topic_id: Option<TopicId>,
}

impl From<TopicInfoQueryData> for AnyQueryData {
    #[inline]
    fn from(data: TopicInfoQueryData) -> Self {
        Self::TopicInfo(data)
    }
}

impl TopicInfoQuery {
    /// Returns the topic to retrieve info about.
    #[must_use]
    pub fn get_topic_id(&self) -> Option<TopicId> {
        self.data.topic_id
    }

    /// Sets the topic to retrieve info about.
    pub fn topic_id(&mut self, id: impl Into<TopicId>) -> &mut Self {
        self.data.topic_id = Some(id.into());
        self
    }
}

impl ToQueryProtobuf for TopicInfoQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let topic_id = self.topic_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::ConsensusGetTopicInfo(
                services::ConsensusGetTopicInfoQuery { topic_id, header: Some(header) },
            )),
        }
    }
}

impl QueryExecute for TopicInfoQueryData {
    type Response = TopicInfo;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { ConsensusServiceClient::new(channel).get_topic_info(request).await })
    }
}

impl ValidateChecksums for TopicInfoQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.topic_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        Hbar,
        TopicId,
        TopicInfoQuery,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    ConsensusGetTopicInfo(
                        ConsensusGetTopicInfoQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            topic_id: Some(
                                TopicId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    topic_num: 5005,
                                },
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &TopicInfoQuery::new()
                .topic_id(TopicId::new(0, 0, 5005))
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        )
    }

    #[test]
    fn get_set_topic_id() {
        let mut query = TopicInfoQuery::new();
        query.topic_id(TopicId::new(0, 0, 5005));

        assert_eq!(query.get_topic_id(), Some(TopicId::new(0, 0, 5005)));
    }
}
