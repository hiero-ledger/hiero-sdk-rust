// SPDX-License-Identifier: Apache-2.0

use crate::fee_estimate::fee_extra::FeeExtra;

/// The fee estimate for a specific component (node or service).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeEstimate {
    /// The base fee price, in tinycents.
    pub base: u64,
    /// The extra fees that apply for this fee component.
    pub extras: Vec<FeeExtra>,
}

impl FeeEstimate {
    /// Create a new `FeeEstimate`.
    pub fn new(base: u64, extras: Vec<FeeExtra>) -> Self {
        Self { base, extras }
    }
}

