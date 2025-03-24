// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;

use crate::{
    AccountId,
    FixedFee,
    FixedFeeData,
    FromProtobuf,
    ToProtobuf,
    TokenId,
};

/// A custom fee definition for a consensus topic.
///
/// This fee definition is specific to an Hedera Consensus Service (HCS) topic
/// and SHOULD NOT be used in any other context.
///
/// All fields for this message are REQUIRED.
///
/// Only "fixed" fee definitions are supported because there is no basis for
/// a fractional fee on a consensus submit transaction.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Default)]
pub struct CustomFixedFee {
    /// The amount of HBAR or other token described by this `FixedFee` SHALL
    /// be charged to the transction payer for each message submitted to a
    /// topic that assigns this consensus custom fee.
    pub amount: u64,

    /// The denomination of the fee; taken as hbar if left unset and, in a TokenCreate, taken as the id
    /// of the newly created token if set to the sentinel value of 0.0.0
    pub denominating_token_id: Option<TokenId>,

    /// The account to receive the custom fee.
    pub fee_collector_account_id: Option<AccountId>,
}

impl CustomFixedFee {
    /// Creates a new `CustomFixedFee`
    pub fn new(
        amount: u64,
        denominating_token_id: Option<TokenId>,
        fee_collector_account_id: Option<AccountId>,
    ) -> Self {
        Self { amount, denominating_token_id, fee_collector_account_id }
    }
}

impl CustomFixedFee {
    pub(crate) fn to_fixed_fee_protobuf(&self) -> services::FixedFee {
        services::FixedFee {
            amount: self.amount as i64,
            denominating_token_id: self.denominating_token_id.to_protobuf(),
        }
    }
}

impl From<CustomFixedFee> for FixedFee {
    fn from(v: CustomFixedFee) -> Self {
        Self {
            fee: FixedFeeData {
                amount: v.amount as i64,
                denominating_token_id: v.denominating_token_id,
            },
            fee_collector_account_id: v.fee_collector_account_id,
            all_collectors_are_exempt: false,
        }
    }
}

impl ToProtobuf for CustomFixedFee {
    type Protobuf = services::FixedCustomFee;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            fixed_fee: Some(services::FixedFee {
                amount: self.amount as i64,
                denominating_token_id: self.denominating_token_id.to_protobuf(),
            }),
            fee_collector_account_id: self.fee_collector_account_id.to_protobuf(),
        }
    }
}

impl FromProtobuf<services::FixedCustomFee> for CustomFixedFee {
    fn from_protobuf(pb: services::FixedCustomFee) -> crate::Result<Self> {
        let fee = FixedFeeData::from_protobuf(pb.fixed_fee.unwrap())?;

        Ok(Self {
            amount: fee.amount as u64,
            denominating_token_id: fee.denominating_token_id,
            fee_collector_account_id: Option::from_protobuf(pb.fee_collector_account_id)?,
        })
    }
}

impl FromProtobuf<services::FixedFee> for CustomFixedFee {
    fn from_protobuf(pb: services::FixedFee) -> crate::Result<Self> {
        let fee = FixedFeeData::from_protobuf(pb)?;

        Ok(Self {
            amount: fee.amount as u64,
            denominating_token_id: fee.denominating_token_id,
            fee_collector_account_id: None,
        })
    }
}
