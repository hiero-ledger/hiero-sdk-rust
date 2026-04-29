// SPDX-License-Identifier: Apache-2.0

use std::fmt;

/// The mode used for fee estimation.
///
/// Determines whether the mirror node uses network state or just the transaction payload
/// to estimate fees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FeeEstimateMode {
    /// Uses the mirror node's latest known state, including exchange rates,
    /// gas prices, token associations, custom fees, and other state-dependent factors.
    State,

    /// Default mode: estimates fees based solely on the transaction payload
    /// (bytes, signatures, keys, gas), ignoring state-dependent factors.
    #[default]
    Intrinsic,
}

impl FeeEstimateMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::State => "STATE",
            Self::Intrinsic => "INTRINSIC",
        }
    }
}

impl fmt::Display for FeeEstimateMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
