// SPDX-License-Identifier: Apache-2.0

//! Conditional protobuf re-exports based on target architecture
//! 
//! This module provides the same protobuf API for both native and WASM builds:
//! - Native: Uses hedera_proto (with tonic/networking)
//! - WASM: Uses hedera_proto_wasm (prost-only, no networking)

// For native builds, re-export everything from hedera-proto
#[cfg(not(target_arch = "wasm32"))]
pub use hedera_proto::*;

// Re-export BoxGrpcFuture at root level for easy access within crate
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use crate::BoxGrpcFuture;

// Native services module with Channel and BoxGrpcFuture re-export
#[cfg(not(target_arch = "wasm32"))]
pub mod services {
    // Re-export everything from hedera_proto::services
    pub use hedera_proto::services::*;
    // Re-export Channel so it's available as services::Channel
    pub use tonic::transport::Channel;
    // Re-export BoxGrpcFuture so it's available as services::BoxGrpcFuture within crate
    pub(crate) use crate::BoxGrpcFuture;
}

// For WASM builds, map hedera-proto-wasm types to the expected paths
#[cfg(target_arch = "wasm32")]
pub mod services {
    // Re-export all the deeply nested types to match hedera_proto::services structure
    pub use hedera_proto_wasm::proto::proto::*;

    // Add the missing addressbook Node transaction types
    pub use hedera_proto_wasm::proto::com::hedera::hapi::node::addressbook::{
        NodeCreateTransactionBody,
        NodeDeleteTransactionBody, 
        NodeUpdateTransactionBody,
    };

    // Special mappings for response and query enums that are deeply nested
    pub mod response {
        pub use hedera_proto_wasm::proto::proto::response::Response;
        // Flatten nested response types for compatibility
        pub use hedera_proto_wasm::proto::proto::response::Response::*;
    }

    pub mod query {
        pub use hedera_proto_wasm::proto::proto::query::Query;
        // Flatten nested query types for compatibility
        pub use hedera_proto_wasm::proto::proto::query::Query::*;
    }
}

#[cfg(target_arch = "wasm32")]
pub mod sdk {
    // TransactionList for SDK compatibility
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct TransactionList {
        #[prost(message, repeated, tag = "1")]
        pub transaction_list: Vec<super::services::Transaction>,
    }
} 