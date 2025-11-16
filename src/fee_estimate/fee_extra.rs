// SPDX-License-Identifier: Apache-2.0

/// The extra fee charged for the transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeExtra {
    /// The charged count of items as calculated by `max(0, count - included)`.
    pub charged: u32,
    /// The actual count of items received.
    pub count: u32,
    /// The fee price per unit in tinycents.
    pub fee_per_unit: u64,
    /// The count of this "extra" that is included for free.
    pub included: u32,
    /// The unique name of this extra fee as defined in the fee schedule.
    pub name: String,
    /// The subtotal in tinycents for this extra fee. Calculated by multiplying the
    /// charged count by the fee_per_unit.
    pub subtotal: u64,
}

impl FeeExtra {
    /// Create a new `FeeExtra`.
    pub fn new(
        charged: u32,
        count: u32,
        fee_per_unit: u64,
        included: u32,
        name: String,
        subtotal: u64,
    ) -> Self {
        Self { charged, count, fee_per_unit, included, name, subtotal }
    }
}
