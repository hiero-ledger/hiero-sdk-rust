// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::consensus_service_client::ConsensusServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    BoxGrpcFuture,
    Error,
    TopicId,
    Transaction,
    ValidateChecksums,
};

/// Delete a topic.
///
/// No more transactions or queries on the topic will succeed.
///
/// If an `admin_key` is set, this transaction must be signed by that key.
/// If there is no `admin_key`, this transaction will fail `UNAUTHORIZED`.
///
pub type TopicDeleteTransaction = Transaction<TopicDeleteTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct TopicDeleteTransactionData {
    /// The topic ID which is being deleted in this transaction.
    topic_id: Option<TopicId>,
}

impl TopicDeleteTransaction {
    /// Returns the ID of the topic which is being deleted in this transaction.
    #[must_use]
    pub fn get_topic_id(&self) -> Option<TopicId> {
        self.data().topic_id
    }

    /// Sets the topic ID which is being deleted in this transaction.
    pub fn topic_id(&mut self, id: impl Into<TopicId>) -> &mut Self {
        self.data_mut().topic_id = Some(id.into());
        self
    }
}

impl TransactionData for TopicDeleteTransactionData {}

impl TransactionExecute for TopicDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { ConsensusServiceClient::new(channel).delete_topic(request).await })
    }
}

impl ValidateChecksums for TopicDeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.topic_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TopicDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::ConsensusDeleteTopic(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for TopicDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::ConsensusDeleteTopic(self.to_protobuf())
    }
}

impl From<TopicDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: TopicDeleteTransactionData) -> Self {
        Self::TopicDelete(transaction)
    }
}

impl FromProtobuf<services::ConsensusDeleteTopicTransactionBody> for TopicDeleteTransactionData {
    fn from_protobuf(pb: services::ConsensusDeleteTopicTransactionBody) -> crate::Result<Self> {
        Ok(Self { topic_id: Option::from_protobuf(pb.topic_id)? })
    }
}

impl ToProtobuf for TopicDeleteTransactionData {
    type Protobuf = services::ConsensusDeleteTopicTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::ConsensusDeleteTopicTransactionBody { topic_id: self.topic_id.to_protobuf() }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyTransaction,
        TopicDeleteTransaction,
        TopicId,
    };

    fn make_transaction() -> TopicDeleteTransaction {
        let mut tx = TopicDeleteTransaction::new_for_tests();

        tx.topic_id("0.0.5007".parse::<TopicId>().unwrap()).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            ConsensusDeleteTopic(
                ConsensusDeleteTopicTransactionBody {
                    topic_id: Some(
                        TopicId {
                            shard_num: 0,
                            realm_num: 0,
                            topic_num: 5007,
                        },
                    ),
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }
}
