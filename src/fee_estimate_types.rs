// SPDX-License-Identifier: Apache-2.0

use std::fmt;

/// Represents an individual extra fee charged beyond the base amount.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeExtra {
    /// Unique name identifier from the fee schedule.
    pub name: Option<String>,

    /// Count of items included for free.
    pub included: u64,

    /// Actual count of items.
    pub count: u64,

    /// Billable item count: `max(0, count - included)`.
    pub charged: u64,

    /// Fee price per unit in tinycents.
    pub fee_per_unit: u64,

    /// Total fee in tinycents: `charged * fee_per_unit`.
    pub subtotal: u64,
}

impl fmt::Display for FeeExtra {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FeeExtra {{ name: {:?}, included: {}, count: {}, charged: {}, fee_per_unit: {}, subtotal: {} }}",
            self.name, self.included, self.count, self.charged, self.fee_per_unit, self.subtotal
        )
    }
}

/// Represents a fee estimate component with a base fee and optional extras.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeEstimate {
    /// Base fee price in tinycents.
    pub base: u64,

    /// Additional fees beyond the base amount.
    pub extras: Vec<FeeExtra>,
}

impl FeeEstimate {
    /// Returns the subtotal: base + sum of all extras' subtotals.
    #[must_use]
    pub fn subtotal(&self) -> u64 {
        self.base + self.extras.iter().map(|e| e.subtotal).sum::<u64>()
    }
}

impl fmt::Display for FeeEstimate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FeeEstimate {{ base: {}, extras: {} }}", self.base, self.extras.len())
    }
}

/// Represents the network fee component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkFee {
    /// Multiplier applied to the node fee subtotal to compute the network fee.
    pub multiplier: u32,

    /// Network fee subtotal in tinycents.
    pub subtotal: u64,
}

impl fmt::Display for NetworkFee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetworkFee {{ multiplier: {}, subtotal: {} }}", self.multiplier, self.subtotal)
    }
}

/// The complete fee estimation response from the mirror node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeEstimateResponse {
    /// High-volume pricing multiplier per HIP-1313. Defaults to 1 when not applicable.
    pub high_volume_multiplier: u64,

    /// Network fee component covering gossip, consensus, signature verification, and storage.
    pub network: Option<NetworkFee>,

    /// Node fee component for node compensation.
    pub node: Option<FeeEstimate>,

    /// Service fee component for execution and state persistence.
    pub service: Option<FeeEstimate>,

    /// Total combined fee in tinycents.
    pub total: u64,
}

impl fmt::Display for FeeEstimateResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FeeEstimateResponse {{ total: {} }}", self.total)
    }
}
