// SPDX-License-Identifier: Apache-2.0

use std::iter;

use time::OffsetDateTime;

use crate::TransactionId;

/// Metadata for an individual chunk
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct TopicMessageChunk {
    /// The consensus timestamp for this chunk.
    pub consensus_timestamp: OffsetDateTime,

    /// How large the content of this specific chunk was.
    pub content_size: usize,

    /// The new running hash of the topic that received the message.
    pub running_hash: Vec<u8>,

    /// Sequence number for this chunk.
    pub sequence_number: u64,
}

/// Topic message records.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct TopicMessage {
    /// The consensus timestamp of the message.
    ///
    /// If there are multiple chunks, this is taken from the *last* chunk.
    pub consensus_timestamp: OffsetDateTime,

    /// The content of the message.
    pub contents: Vec<u8>,

    /// The new running hash of the topic that received the message.
    ///
    /// If there are multiple chunks, this is taken from the *last* chunk.
    pub running_hash: Vec<u8>,

    /// Version of the SHA-384 digest used to update the running hash.
    ///
    /// If there are multiple chunks, this is taken from the *last* chunk.
    pub running_hash_version: u64,

    /// The sequence number of the message relative to all other messages
    /// for the same topic.
    ///
    /// If there are multiple chunks, this is taken from the *last* chunk.
    pub sequence_number: u64,

    /// The chunks that make up this message.
    pub chunks: Option<Vec<TopicMessageChunk>>,

    /// The [`TransactionId`] of the first chunk, gets copied to every subsequent chunk in the message.
    pub transaction: Option<TransactionId>,
}

impl TopicMessage {
    pub(crate) fn from_single(pb: PbTopicMessageHeader) -> Self {
        Self {
            consensus_timestamp: pb.consensus_timestamp,
            contents: pb.message,
            running_hash: pb.running_hash,
            running_hash_version: pb.running_hash_version,
            sequence_number: pb.sequence_number,
            chunks: None,
            transaction: None,
        }
    }

    pub(crate) fn from_chunks(pb: Vec<PbTopicMessageChunk>) -> Self {
        assert!(!pb.is_empty(), "no chunks provided to `TopicMessage::from_chunks`");

        if log::log_enabled!(log::Level::Warn) {
            let (first, rest) = pb.split_first().unwrap();

            if !rest.iter().all(|it| first.total == it.total) {
                log::warn!("`TopicMessageChunk` mismatched totals (ignoring)");
            }

            let all_ascending_no_gaps = pb.iter().all({
                let mut current = 1;
                move |it| {
                    let res = it.number == current;
                    current += 1;

                    res
                }
            });

            if !all_ascending_no_gaps {
                log::warn!("`TopicMessageChunk` mismatched numbers (ignoring)");
                // return Err(Error::from_protobuf("`TopicMessageChunk` mismatched numbers"));
            }
        }

        let contents = pb.iter().fold(Vec::new(), |mut acc, it| {
            acc.extend_from_slice(&it.header.message);
            acc
        });

        let mut pb = pb;

        let last = pb.pop().unwrap();

        let chunks = pb
            .into_iter()
            .map(|it| TopicMessageChunk {
                consensus_timestamp: it.header.consensus_timestamp,
                content_size: it.header.message.len(),
                running_hash: it.header.running_hash,
                sequence_number: it.header.sequence_number,
            })
            .chain(iter::once(TopicMessageChunk {
                consensus_timestamp: last.header.consensus_timestamp,
                content_size: last.header.message.len(),
                running_hash: last.header.running_hash.clone(),
                sequence_number: last.header.sequence_number,
            }))
            .collect();

        Self {
            consensus_timestamp: last.header.consensus_timestamp,
            contents,
            running_hash: last.header.running_hash,
            running_hash_version: last.header.running_hash_version,
            sequence_number: last.header.sequence_number,
            chunks: Some(chunks),
            transaction: Some(last.initial_transaction_id),
        }
    }
}

pub(crate) struct PbTopicMessageHeader {
    pub(crate) consensus_timestamp: OffsetDateTime,
    pub(crate) sequence_number: u64,
    pub(crate) running_hash: Vec<u8>,
    pub(crate) running_hash_version: u64,
    pub(crate) message: Vec<u8>,
}

pub(crate) struct PbTopicMessageChunk {
    pub(crate) header: PbTopicMessageHeader,
    pub(crate) initial_transaction_id: TransactionId,
    pub(crate) number: i32,
    pub(crate) total: i32,
}
