// SPDX-License-Identifier: Apache-2.0

use std::cmp;
use std::num::NonZeroUsize;

#[cfg(not(target_arch = "wasm32"))]
use hedera_proto::services::consensus_service_client::ConsensusServiceClient;
#[cfg(not(target_arch = "wasm32"))]
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::proto::services;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::transaction::ChunkData;
#[cfg(not(target_arch = "wasm32"))]
use crate::transaction::ChunkedTransactionData;
#[cfg(not(target_arch = "wasm32"))]
use crate::transaction::TransactionExecute;
#[cfg(not(target_arch = "wasm32"))]
use crate::transaction::TransactionExecuteChunked;
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
};
use crate::{
    Error,
    TopicId,
    Transaction,
    ValidateChecksums,
};

/// Submit a message for consensus.
///
/// Valid and authorized messages on valid topics will be ordered by the consensus service, gossipped to the
/// mirror net, and published (in order) to all subscribers (from the mirror net) on this topic.
///
/// The `submit_key` (if any) must sign this transaction.
///
/// On success, the resulting `TransactionReceipt` contains the topic's updated `topic_sequence_number` and
/// `topic_running_hash`.
///
pub type TopicMessageSubmitTransaction = Transaction<TopicMessageSubmitTransactionData>;

#[derive(Debug, Default, Clone)]
pub struct TopicMessageSubmitTransactionData {
    /// The topic ID to submit this message to.
    topic_id: Option<TopicId>,

    #[cfg(not(target_arch = "wasm32"))]
    chunk_data: ChunkData,
}

impl TopicMessageSubmitTransaction {
    /// Returns the ID of the topic this message will be submitted to.
    #[must_use]
    pub fn get_topic_id(&self) -> Option<TopicId> {
        self.data().topic_id
    }

    /// Sets the topic ID to submit this message to.
    pub fn topic_id(&mut self, id: impl Into<TopicId>) -> &mut Self {
        self.data_mut().topic_id = Some(id.into());
        self
    }

    /// Returns the message to be submitted.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_message(&self) -> Option<&[u8]> {
        Some(self.data().chunk_data.data.as_slice())
    }

    /// Sets the message to be submitted.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn message(&mut self, bytes: impl Into<Vec<u8>>) -> &mut Self {
        self.data_mut().chunk_data_mut().data = bytes.into();
        self
    }
}

impl TransactionData for TopicMessageSubmitTransactionData {
    #[cfg(not(target_arch = "wasm32"))]
    fn maybe_chunk_data(&self) -> Option<&ChunkData> {
        Some(self.chunk_data())
    }

    fn wait_for_receipt(&self) -> bool {
        false
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ChunkedTransactionData for TopicMessageSubmitTransactionData {
    fn chunk_data(&self) -> &ChunkData {
        &self.chunk_data
    }

    fn chunk_data_mut(&mut self) -> &mut ChunkData {
        &mut self.chunk_data
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TransactionExecute for TopicMessageSubmitTransactionData {
    fn execute(
        &self,
        channel: services::Channel,
        request: services::Transaction,
    ) -> services::BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { ConsensusServiceClient::new(channel).submit_message(request).await })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TransactionExecuteChunked for TopicMessageSubmitTransactionData {}

impl ValidateChecksums for TopicMessageSubmitTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.topic_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for TopicMessageSubmitTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        services::transaction_body::Data::ConsensusSubmitMessage(
            services::ConsensusSubmitMessageTransactionBody {
                topic_id: self.topic_id.to_protobuf(),
                #[cfg(not(target_arch = "wasm32"))]
                message: self.chunk_data.message_chunk(chunk_info).to_vec(),
                #[cfg(target_arch = "wasm32")]
                message: Vec::new(), // WASM doesn't support chunked transactions
                chunk_info: (chunk_info.total > 1).then(|| services::ConsensusMessageChunkInfo {
                    initial_transaction_id: Some(chunk_info.initial_transaction_id.to_protobuf()),
                    number: (chunk_info.current + 1) as i32,
                    total: chunk_info.total as i32,
                }),
            },
        )
    }
}

impl ToSchedulableTransactionDataProtobuf for TopicMessageSubmitTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        #[cfg(not(target_arch = "wasm32"))]
        assert!(
            self.chunk_data.used_chunks() == 1,
            "Cannot schedule a `TopicMessageSubmitTransaction` with multiple chunks"
        );

        let data = services::ConsensusSubmitMessageTransactionBody {
            topic_id: self.topic_id.to_protobuf(),
            #[cfg(not(target_arch = "wasm32"))]
            message: self.chunk_data.data.clone(),
            #[cfg(target_arch = "wasm32")]
            message: Vec::new(), // WASM doesn't support chunked transactions
            chunk_info: None,
        };

        services::schedulable_transaction_body::Data::ConsensusSubmitMessage(data)
    }
}

impl From<TopicMessageSubmitTransactionData> for AnyTransactionData {
    fn from(transaction: TopicMessageSubmitTransactionData) -> Self {
        Self::TopicMessageSubmit(transaction)
    }
}

impl FromProtobuf<services::ConsensusSubmitMessageTransactionBody>
    for TopicMessageSubmitTransactionData
{
    fn from_protobuf(pb: services::ConsensusSubmitMessageTransactionBody) -> crate::Result<Self> {
        Self::from_protobuf(Vec::from([pb]))
    }
}

impl FromProtobuf<Vec<services::ConsensusSubmitMessageTransactionBody>>
    for TopicMessageSubmitTransactionData
{
    fn from_protobuf(
        pb: Vec<services::ConsensusSubmitMessageTransactionBody>,
    ) -> crate::Result<Self> {
        let total_chunks = pb.len();

        let mut iter = pb.into_iter();
        let pb_first = iter.next().expect("Empty transaction (should've been handled earlier)");

        let topic_id = Option::from_protobuf(pb_first.topic_id)?;

        let mut largest_chunk_size = pb_first.message.len();
        let mut message = pb_first.message;

        // note: no other SDK checks for correctness here... so let's not do it here either?

        for item in iter {
            largest_chunk_size = cmp::max(largest_chunk_size, item.message.len());
            message.extend_from_slice(&item.message);
        }

        Ok(Self {
            topic_id,
            #[cfg(not(target_arch = "wasm32"))]
            chunk_data: ChunkData {
                max_chunks: total_chunks,
                chunk_size: NonZeroUsize::new(largest_chunk_size)
                    .unwrap_or_else(|| NonZeroUsize::new(1).unwrap()),
                data: message,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::transaction::test_helpers::{
        check_body,
        transaction_bodies,
    };
    use crate::{
        AnyTransaction,
        TopicId,
        TopicMessageSubmitTransaction,
    };

    const TOPIC_ID: TopicId = TopicId::new(0, 0, 10);

    const MESSAGE: &[u8] = br#"{"foo": 231}"#;

    fn make_transaction() -> TopicMessageSubmitTransaction {
        let mut tx = TopicMessageSubmitTransaction::new_for_tests();
        tx.topic_id(TOPIC_ID).message(MESSAGE).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        // unlike most transactions we *do* need to do this like in case it's chunked.
        // granted, trying to do anything with a chunked transaction without a Client is hard.
        let txes = transaction_bodies(tx);

        // this is kinda a mess... But it works.
        let txes: Vec<_> = txes.into_iter().map(check_body).collect();

        expect![[r#"
            [
                ConsensusSubmitMessage(
                    ConsensusSubmitMessageTransactionBody {
                        topic_id: Some(
                            TopicId {
                                shard_num: 0,
                                realm_num: 0,
                                topic_num: 10,
                            },
                        ),
                        message: [
                            123,
                            34,
                            102,
                            111,
                            111,
                            34,
                            58,
                            32,
                            50,
                            51,
                            49,
                            125,
                        ],
                        chunk_info: None,
                    },
                ),
                ConsensusSubmitMessage(
                    ConsensusSubmitMessageTransactionBody {
                        topic_id: Some(
                            TopicId {
                                shard_num: 0,
                                realm_num: 0,
                                topic_num: 10,
                            },
                        ),
                        message: [
                            123,
                            34,
                            102,
                            111,
                            111,
                            34,
                            58,
                            32,
                            50,
                            51,
                            49,
                            125,
                        ],
                        chunk_info: None,
                    },
                ),
            ]
        "#]]
        .assert_debug_eq(&txes);
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_bodies(tx);
        let tx2 = transaction_bodies(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn get_set_topic_id() {
        let mut tx = TopicMessageSubmitTransaction::new();
        tx.topic_id(TOPIC_ID);

        assert_eq!(tx.get_topic_id(), Some(TOPIC_ID));
    }

    #[test]
    fn get_set_message() {
        let mut tx = TopicMessageSubmitTransaction::new();
        tx.message(MESSAGE);

        assert_eq!(tx.get_message(), Some(MESSAGE));
    }

    #[test]
    #[should_panic]
    fn get_set_topic_id_frozen_panics() {
        let mut tx = make_transaction();
        tx.topic_id(TOPIC_ID);
    }

    #[test]
    #[should_panic]
    fn get_set_message_frozen_panics() {
        let mut tx = make_transaction();
        tx.message(MESSAGE);
    }
}
