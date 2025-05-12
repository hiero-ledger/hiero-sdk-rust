// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use hedera_proto::services;

use crate::ledger_id::RefLedgerId;
use crate::{
    AccountId,
    Error,
    FromProtobuf,
    NftId,
    ToProtobuf,
    TokenId,
    ValidateChecksums,
};

/// A unique, composite, identifier for a pending airdrop.
///
/// Each pending airdrop SHALL be uniquely identified by a PendingAirdropId.
/// A PendingAirdropId SHALL be recorded when created and MUST be provided in any transaction
/// that would modify that pending airdrop (such as a `claimAirdrop` or `cancelAirdrop`).
///
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct PendingAirdropId {
    /// A sending account.
    ///
    /// This is the account that initiated, and SHALL fund, this pending airdrop.
    /// This field is REQUIRED.
    pub sender_id: AccountId,

    /// A receiving account.
    ///
    /// This is the ID of the account that SHALL receive the airdrop.
    /// This field is REQUIRED.
    pub receiver_id: AccountId,

    /// Token Id.
    pub token_id: Option<TokenId>,

    /// Nft Id.
    pub nft_id: Option<NftId>,
}

impl fmt::Debug for PendingAirdropId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingAirdropId")
            .field("sender_id", &self.sender_id)
            .field("receiver_id", &self.receiver_id)
            .field("token_id", &self.token_id)
            .field("nft_id", &self.nft_id)
            .finish()
    }
}

impl fmt::Display for PendingAirdropId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PendingAirdropId {{ sender_id: {}, receiver_id: {}, token_id: {}, nft_id: {} }}",
            self.sender_id,
            self.receiver_id,
            self.token_id.map_or_else(|| "None".to_string(), |v| v.to_string()),
            self.nft_id.map_or_else(|| "None".to_string(), |v| v.to_string()),
        )
    }
}

impl PendingAirdropId {
    pub const fn new_nft_id(sender_id: AccountId, receiver_id: AccountId, nft_id: NftId) -> Self {
        Self { sender_id, receiver_id, token_id: None, nft_id: Some(nft_id) }
    }

    pub const fn new_token_id(
        sender_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
    ) -> Self {
        Self { sender_id, receiver_id, token_id: Some(token_id), nft_id: None }
    }

    /// Create a new `PendingAirdropId` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl ValidateChecksums for PendingAirdropId {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        let _ = self.sender_id.validate_checksums(ledger_id);
        let _ = self.receiver_id.validate_checksums(ledger_id);

        if let Some(token_id) = self.token_id {
            token_id.validate_checksums(ledger_id)?
        };

        if let Some(nft_id) = &self.nft_id {
            nft_id.validate_checksums(ledger_id)?
        }

        Ok(())
    }
}

impl FromProtobuf<services::PendingAirdropId> for PendingAirdropId {
    fn from_protobuf(pb: services::PendingAirdropId) -> crate::Result<Self> {
        let sender_id = AccountId::from_protobuf(pb_getf!(pb, sender_id)?)?;
        let receiver_id = AccountId::from_protobuf(pb_getf!(pb, receiver_id)?)?;

        let nft_id = if let Some(reference) = pb.token_reference.clone() {
            match reference {
                services::pending_airdrop_id::TokenReference::NonFungibleToken(nft_id) => {
                    Some(NftId::from_protobuf(nft_id)?)
                }
                _ => None,
            }
        } else {
            None
        };

        let token_id = if let Some(token) = pb.token_reference {
            match token {
                services::pending_airdrop_id::TokenReference::FungibleTokenType(token_id) => {
                    Some(TokenId::from_protobuf(token_id)?)
                }
                _ => None,
            }
        } else {
            None
        };

        Ok(Self { sender_id, receiver_id, token_id, nft_id })
    }
}

impl ToProtobuf for PendingAirdropId {
    type Protobuf = services::PendingAirdropId;

    fn to_protobuf(&self) -> Self::Protobuf {
        let nft_id = self.nft_id.as_ref().map(|nft_id| nft_id.to_protobuf());
        let token_id = self.token_id.as_ref().map(|token_id| token_id.to_protobuf());

        let token_reference = if let Some(nft_id) = nft_id {
            Some(services::pending_airdrop_id::TokenReference::NonFungibleToken(nft_id))
        } else if let Some(token_id) = token_id {
            Some(services::pending_airdrop_id::TokenReference::FungibleTokenType(token_id))
        } else {
            None
        };

        services::PendingAirdropId {
            sender_id: Some(self.sender_id.to_protobuf()),
            receiver_id: Some(self.receiver_id.to_protobuf()),
            token_reference,
        }
    }
}
