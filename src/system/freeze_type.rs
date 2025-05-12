// SPDX-License-Identifier: Apache-2.0

// todo(sr): Not happy with this doc.
/// What type of freeze should be executed?
#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FreezeType {
    /// An (invalid) default value for this enum, to ensure the client explicitly sets
    /// the intended type of freeze transaction.
    #[default]
    Unknown = 0,

    /// Freezes the network at the specified time. The start_time field must be provided and
    /// must reference a future time. Any values specified for the update_file and file_hash
    /// fields will be ignored. This transaction does not perform any network changes or
    /// upgrades and requires manual intervention to restart the network.
    FreezeOnly = 1,

    /// A non-freezing operation that initiates network wide preparation in advance of a
    /// scheduled freeze upgrade. The update_file and file_hash fields must be provided and
    /// valid. The start_time field may be omitted and any value present will be ignored.
    PrepareUpgrade = 2,

    /// Freezes the network at the specified time and performs the previously prepared
    /// automatic upgrade across the entire network.
    FreezeUpgrade = 3,

    /// Aborts a pending network freeze operation.
    FreezeAbort = 4,

    /// Performs an immediate upgrade on auxilary services and containers providing
    /// telemetry/metrics. Does not impact network operations.
    TelemetryUpgrade = 5,
}

impl From<i32> for FreezeType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::FreezeOnly,
            2 => Self::PrepareUpgrade,
            3 => Self::FreezeUpgrade,
            4 => Self::FreezeAbort,
            5 => Self::TelemetryUpgrade,
            ..=0 | 6.. => Self::Unknown,
        }
    }
}
