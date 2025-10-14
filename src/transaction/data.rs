// SPDX-License-Identifier: Apache-2.0

//! Transaction data trait and related types that are WASM-compatible
//!
//! This module contains the core transaction building logic that doesn't require networking.

// Re-export ChunkData from chunked module only for native builds
#[cfg(not(target_arch = "wasm32"))]
use super::ChunkData;
// AnyTransactionData is now available for both WASM and native
// It's just a data enum, not network-dependent
pub use crate::transaction::any::AnyTransactionData;
use crate::{
    AccountId,
    Hbar,
    TransactionId,
};

/// WASM-compatible ChunkInfo that contains essential transaction metadata
///
/// Unlike the native ChunkInfo which handles complex chunked execution,
/// this version only carries the basic metadata needed for transaction building.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub current: usize,
    pub total: usize,
    pub initial_transaction_id: TransactionId,
    pub current_transaction_id: TransactionId,
    pub node_account_id: AccountId,
}

#[cfg(target_arch = "wasm32")]
impl ChunkInfo {
    /// Create a new ChunkInfo for single (non-chunked) transactions
    pub fn new_single(transaction_id: TransactionId, node_account_id: AccountId) -> Self {
        Self {
            current: 0,
            total: 1,
            initial_transaction_id: transaction_id,
            current_transaction_id: transaction_id,
            node_account_id,
        }
    }

    /// Assert this is a single transaction (not chunked)
    ///
    /// For WASM builds, we expect only single transactions since
    /// chunked execution requires networking capabilities.
    pub fn assert_single_transaction(&self) {
        assert_eq!(self.total, 1, "WASM builds only support single transactions");
        assert_eq!(self.current, 0, "WASM builds only support single transactions");
    }
}

/// Core transaction data trait that defines transaction building behavior.
///
/// This trait is available for both native and WASM builds as it contains
/// only transaction construction logic, no networking.

// Native version with AnyTransactionData conversion
#[cfg(not(target_arch = "wasm32"))]
pub trait TransactionData: Clone + Into<AnyTransactionData> {
    /// Whether this transaction is intended to be executed to return a cost estimate.
    #[doc(hidden)]
    fn for_cost_estimate(&self) -> bool {
        false
    }

    /// Returns the maximum allowed transaction fee if none is specified.
    ///
    /// Specifically, this default will be used in the following case:
    /// - The transaction itself (direct user input) has no `max_transaction_fee` specified, AND
    /// - The [`Client`](crate::Client) has no `max_transaction_fee` specified.
    fn default_max_transaction_fee(&self) -> Hbar {
        Hbar::new(2)
    }

    /// Returns the chunk data for this transaction if this is a chunked transaction.
    ///
    /// Note: For WASM builds, this will always return None since chunked execution
    /// requires networking capabilities.
    fn maybe_chunk_data(&self) -> Option<&ChunkData> {
        None
    }

    /// Returns `true` if `self` is a chunked transaction *and* it should wait for receipts between each chunk.
    ///
    /// For WASM builds, this always returns false since chunked execution requires networking.
    fn wait_for_receipt(&self) -> bool {
        false
    }
}

// WASM version without AnyTransactionData conversion (simplified)
#[cfg(target_arch = "wasm32")]
pub trait TransactionData: Clone {
    /// Whether this transaction is intended to be executed to return a cost estimate.
    #[doc(hidden)]
    fn for_cost_estimate(&self) -> bool {
        false
    }

    /// Returns the maximum allowed transaction fee if none is specified.
    ///
    /// Specifically, this default will be used in the following case:
    /// - The transaction itself (direct user input) has no `max_transaction_fee` specified, AND
    /// - The [`Client`](crate::Client) has no `max_transaction_fee` specified.
    fn default_max_transaction_fee(&self) -> Hbar {
        Hbar::new(2)
    }

    /// WASM-compatible version that always returns None
    fn maybe_chunk_data(&self) -> Option<()> {
        None
    }

    /// Returns `true` if `self` is a chunked transaction *and* it should wait for receipts between each chunk.
    ///
    /// For WASM builds, this always returns false since chunked execution requires networking.
    fn wait_for_receipt(&self) -> bool {
        false
    }
}
