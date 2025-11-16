// SPDX-License-Identifier: Apache-2.0

use crate::fee_estimate::{
    fee_estimate::FeeEstimate,
    fee_estimate_mode::FeeEstimateMode,
    network_fee::NetworkFee,
};

/// The response containing the estimated transaction fees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeEstimateResponse {
    /// The mode that was used to calculate the fees.
    pub mode: FeeEstimateMode,
    /// The network fee component which covers the cost of gossip, consensus,
    /// signature verifications, fee payment, and storage.
    pub network: NetworkFee,
    /// The node fee component which is to be paid to the node that submitted the
    /// transaction to the network.
    pub node: FeeEstimate,
    /// An array of strings for any caveats.
    pub notes: Vec<String>,
    /// The service fee component which covers execution costs, state saved in the
    /// Merkle tree, and additional costs to the blockchain storage.
    pub service: FeeEstimate,
    /// The sum of the network, node, and service subtotals in tinycents.
    pub total: u64,
}

impl FeeEstimateResponse {
    /// Create a new `FeeEstimateResponse`.
    pub fn new(
        mode: FeeEstimateMode,
        network: NetworkFee,
        node: FeeEstimate,
        notes: Vec<String>,
        service: FeeEstimate,
        total: u64,
    ) -> Self {
        Self { mode, network, node, notes, service, total }
    }
}

