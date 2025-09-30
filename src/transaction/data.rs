// SPDX-License-Identifier: Apache-2.0

//! Transaction data trait and related types that are WASM-compatible
//! 
//! This module contains the core transaction building logic that doesn't require networking.

use crate::{Hbar};

// For WASM, we need a simplified version of AnyTransactionData
#[cfg(target_arch = "wasm32")]
pub struct AnyTransactionData;

#[cfg(not(target_arch = "wasm32"))]
use crate::transaction::any::AnyTransactionData;

// Re-export ChunkData from chunked module only for native builds
#[cfg(not(target_arch = "wasm32"))]
use super::ChunkData;

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