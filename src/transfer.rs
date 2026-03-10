use hiero_sdk_proto::services;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::{
    AccountId,
    Hbar,
};

/// A transfer of [`Hbar`] that occured within a [`Transaction`](crate::Transaction)
///
/// Returned as part of a [`TransactionRecord`](crate::TransactionRecord)
#[derive(Debug, Clone)]
pub struct Transfer {
    /// The account ID that this transfer is to/from.
    pub account_id: AccountId,

    /// The value of this transfer.
    ///
    /// Negative if the account sends/withdraws hbar, positive if it receives hbar.
    pub amount: Hbar,
}

impl FromProtobuf<services::AccountAmount> for Transfer {
    fn from_protobuf(pb: services::AccountAmount) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            account_id: AccountId::from_protobuf(pb_getf!(pb, account_id)?)?,
            amount: Hbar::from_tinybars(pb.amount),
        })
    }
}

impl ToProtobuf for Transfer {
    type Protobuf = services::AccountAmount;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::AccountAmount {
            account_id: Some(self.account_id.to_protobuf()),
            amount: self.amount.to_tinybars(),
            is_approval: false,
        }
    }
}
