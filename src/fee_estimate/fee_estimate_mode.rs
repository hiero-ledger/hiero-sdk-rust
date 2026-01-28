// SPDX-License-Identifier: Apache-2.0

/// The mode for fee estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FeeEstimateMode {
    /// Estimate based on intrinsic properties plus the latest known state.
    State = 0,
    /// Estimate based solely on the transaction's inherent properties.
    Intrinsic = 1,
}

impl FeeEstimateMode {
    /// Returns the numeric value of the mode.
    pub const fn value(self) -> u32 {
        self as u32
    }

    /// Creates a `FeeEstimateMode` from a numeric value.
    pub const fn from_value(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::State),
            1 => Some(Self::Intrinsic),
            _ => None,
        }
    }
}

impl Default for FeeEstimateMode {
    fn default() -> Self {
        Self::State
    }
}

impl From<FeeEstimateMode> for u32 {
    fn from(mode: FeeEstimateMode) -> Self {
        mode.value()
    }
}

impl TryFrom<u32> for FeeEstimateMode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_value(value).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_estimate_mode_values() {
        assert_eq!(FeeEstimateMode::State.value(), 0);
        assert_eq!(FeeEstimateMode::Intrinsic.value(), 1);
    }

    #[test]
    fn test_fee_estimate_mode_from_value() {
        assert_eq!(FeeEstimateMode::from_value(0), Some(FeeEstimateMode::State));
        assert_eq!(FeeEstimateMode::from_value(1), Some(FeeEstimateMode::Intrinsic));
        assert_eq!(FeeEstimateMode::from_value(2), None);
    }
}
