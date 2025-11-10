use hedera_proto::services;

use crate::hooks::{
    NftHookCall,
    NftHookType,
};
use crate::protobuf::FromProtobuf;
use crate::{
    AccountId,
    TokenId,
};

/// Represents a transfer of an NFT from one account to another.
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct TokenNftTransfer {
    /// The ID of the NFT's token.
    pub token_id: TokenId,

    /// The account that the NFT is being transferred from.
    pub sender: AccountId,

    /// The account that the NFT is being transferred to.
    pub receiver: AccountId,

    /// The serial number for the NFT being transferred.
    pub serial: u64,

    /// If true then the transfer is expected to be an approved allowance and the
    /// `sender` is expected to be the owner. The default is false.
    pub is_approved: bool,

    /// Optional hook call for the sender side of this NFT transfer.
    pub sender_hook_call: Option<NftHookCall>,

    /// Optional hook call for the receiver side of this NFT transfer.
    pub receiver_hook_call: Option<NftHookCall>,
}

impl TokenNftTransfer {
    pub(crate) fn from_protobuf(
        pb: services::NftTransfer,
        token_id: TokenId,
    ) -> crate::Result<Self> {
        // Extract sender hook call from the oneof union
        let sender_hook_call = match pb.sender_allowance_hook_call {
            Some(services::nft_transfer::SenderAllowanceHookCall::PreTxSenderAllowanceHook(
                hook,
            )) => Some(NftHookCall::from_protobuf_with_type(hook, NftHookType::PreHookSender)?),
            Some(
                services::nft_transfer::SenderAllowanceHookCall::PrePostTxSenderAllowanceHook(hook),
            ) => Some(NftHookCall::from_protobuf_with_type(hook, NftHookType::PrePostHookSender)?),
            None => None,
        };

        // Extract receiver hook call from the oneof union
        let receiver_hook_call = match pb.receiver_allowance_hook_call {
            Some(
                services::nft_transfer::ReceiverAllowanceHookCall::PreTxReceiverAllowanceHook(hook),
            ) => Some(NftHookCall::from_protobuf_with_type(hook, NftHookType::PreHookReceiver)?),
            Some(
                services::nft_transfer::ReceiverAllowanceHookCall::PrePostTxReceiverAllowanceHook(
                    hook,
                ),
            ) => {
                Some(NftHookCall::from_protobuf_with_type(hook, NftHookType::PrePostHookReceiver)?)
            }
            None => None,
        };

        Ok(Self {
            token_id,
            sender: AccountId::from_protobuf(pb_getf!(pb, sender_account_id)?)?,
            receiver: AccountId::from_protobuf(pb_getf!(pb, receiver_account_id)?)?,
            serial: pb.serial_number as u64,
            is_approved: pb.is_approval,
            sender_hook_call,
            receiver_hook_call,
        })
    }
}
