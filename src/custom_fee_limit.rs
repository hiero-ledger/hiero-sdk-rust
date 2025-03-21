use hedera_proto::services::{
    self,
};

use crate::custom_fixed_fee::CustomFixedFee;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::AccountId;

/// A custom transfer fee that was assessed during the handling of a `CryptoTransfer`.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CustomFeeLimit {
    /// The account of the fee payer
    pub account_id: Option<AccountId>,

    /// The maximum fees that the user is willing to pay for the message.
    pub fees: Vec<CustomFixedFee>,
}

impl CustomFeeLimit {
    pub fn new(account_id: Option<AccountId>, fees: Vec<CustomFixedFee>) -> Self {
        Self { account_id, fees }
    }
}

impl FromProtobuf<services::CustomFeeLimit> for CustomFeeLimit {
    fn from_protobuf(pb: services::CustomFeeLimit) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut fees = Vec::new();
        for fee in pb.fees {
            fees.push(CustomFixedFee {
                amount: fee.amount as u64,
                denominating_token_id: Option::from_protobuf(fee.denominating_token_id)?,
                fee_collector_account_id: None,
            });
        }

        Ok(Self { account_id: Option::from_protobuf(pb.account_id)?, fees })
    }
}

impl ToProtobuf for CustomFeeLimit {
    type Protobuf = services::CustomFeeLimit;

    fn to_protobuf(&self) -> Self::Protobuf {
        let fees: Vec<services::FixedFee> =
            self.fees.iter().map(|fee| fee.to_fixed_fee_protobuf()).collect();

        services::CustomFeeLimit { account_id: self.account_id.to_protobuf(), fees }
    }
}
