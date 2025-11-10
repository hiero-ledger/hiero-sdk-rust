use hedera_proto::services;

use crate::hooks::{
    FungibleHookCall,
    FungibleHookType,
};
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

    /// If true then the transfer is expected to be an approved allowance.
    pub is_approved: bool,

    /// Optional hook call for this transfer.
    pub hook_call: Option<FungibleHookCall>,
}

impl FromProtobuf<services::AccountAmount> for Transfer {
    fn from_protobuf(pb: services::AccountAmount) -> crate::Result<Self>
    where
        Self: Sized,
    {
        // Determine which hook type is present, if any
        let hook_call = match pb.hook_call {
            Some(services::account_amount::HookCall::PreTxAllowanceHook(hook)) => {
                Some(FungibleHookCall::from_protobuf_with_type(
                    hook,
                    FungibleHookType::PreTxAllowanceHook,
                )?)
            }
            Some(services::account_amount::HookCall::PrePostTxAllowanceHook(hook)) => {
                Some(FungibleHookCall::from_protobuf_with_type(
                    hook,
                    FungibleHookType::PrePostTxAllowanceHook,
                )?)
            }
            None => None,
        };

        Ok(Self {
            account_id: AccountId::from_protobuf(pb_getf!(pb, account_id)?)?,
            amount: Hbar::from_tinybars(pb.amount),
            is_approved: pb.is_approval,
            hook_call,
        })
    }
}

impl ToProtobuf for Transfer {
    type Protobuf = services::AccountAmount;

    fn to_protobuf(&self) -> Self::Protobuf {
        let hook_call = self.hook_call.as_ref().map(|hook| match hook.hook_type {
            FungibleHookType::PreTxAllowanceHook => {
                services::account_amount::HookCall::PreTxAllowanceHook(hook.to_protobuf())
            }
            FungibleHookType::PrePostTxAllowanceHook => {
                services::account_amount::HookCall::PrePostTxAllowanceHook(hook.to_protobuf())
            }
        });

        services::AccountAmount {
            account_id: Some(self.account_id.to_protobuf()),
            amount: self.amount.to_tinybars(),
            is_approval: self.is_approved,
            hook_call,
        }
    }
}
