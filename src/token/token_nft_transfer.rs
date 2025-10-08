use crate::proto::services;
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
}

impl TokenNftTransfer {
    pub(crate) fn from_protobuf(
        pb: services::NftTransfer,
        token_id: TokenId,
    ) -> crate::Result<Self> {
        Ok(Self {
            token_id,
            sender: AccountId::from_protobuf(pb_getf!(pb, sender_account_id)?)?,
            receiver: AccountId::from_protobuf(pb_getf!(pb, receiver_account_id)?)?,
            serial: pb.serial_number as u64,
            is_approved: pb.is_approval,
        })
    }
}
