// SPDX-License-Identifier: Apache-2.0

/// The network fee component which covers the cost of gossip, consensus,
/// signature verifications, fee payment, and storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkFee {
    /// Multiplied by the node fee to determine the total network fee.
    pub multiplier: u32,
    /// The subtotal in tinycents for the network fee component which is calculated by
    /// multiplying the node subtotal by the network multiplier.
    pub subtotal: u64,
}

impl NetworkFee {
    /// Create a new `NetworkFee`.
    pub fn new(multiplier: u32, subtotal: u64) -> Self {
        Self { multiplier, subtotal }
    }
}

