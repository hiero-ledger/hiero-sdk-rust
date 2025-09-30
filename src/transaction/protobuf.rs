// SPDX-License-Identifier: Apache-2.0

use crate::proto::services;

// ChunkInfo available for both native and WASM (contains essential metadata)
#[cfg(not(target_arch = "wasm32"))]
use super::chunked::ChunkInfo;
#[cfg(target_arch = "wasm32")]  
use super::data::ChunkInfo;

// Unified trait - ChunkInfo contains essential metadata for both native and WASM
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
