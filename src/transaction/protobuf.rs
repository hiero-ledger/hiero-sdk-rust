// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;

use super::chunked::ChunkInfo;

pub trait ToTransactionDataProtobuf: Send + Sync {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data;
}

pub trait ToSchedulableTransactionDataProtobuf: Send + Sync {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data;
}
