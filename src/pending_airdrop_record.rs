// SPDX-License-Identifier: Apache-2.0

use core::fmt;

use hedera_proto::services;

use crate::pending_airdrop_id::PendingAirdropId;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};

/// A record of a new pending airdrop.
#[derive(Clone)]
pub struct PendingAirdropRecord {
    /// A unique, composite, identifier for a pending airdrop.
    /// This field is REQUIRED.
    pub pending_airdrop_id: PendingAirdropId,

    /// A single pending airdrop amount.
    pub pending_airdrop_value: Option<u64>,
}

impl fmt::Debug for PendingAirdropRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingAirdropRecord")
            .field("pending_airdrop_id", &self.pending_airdrop_id)
            .field("pending_airdrop_value", &self.pending_airdrop_value)
            .finish()
    }
}

impl fmt::Display for PendingAirdropRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PendingAirdropRecord {{ id: {}, value: {} }}",
            self.pending_airdrop_id,
            self.pending_airdrop_value.map_or_else(|| "None".to_string(), |v| v.to_string())
        )
    }
}

impl PendingAirdropRecord {
    /// Create a new `NodeAddressBook` from protobuf-encoded `bytes`.
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

impl FromProtobuf<services::PendingAirdropRecord> for PendingAirdropRecord {
    fn from_protobuf(pb: services::PendingAirdropRecord) -> crate::Result<Self> {
        let airdrop_id = PendingAirdropId::from_protobuf(pb_getf!(pb, pending_airdrop_id)?)?;
        Ok(Self {
            pending_airdrop_id: airdrop_id,
            pending_airdrop_value: pb.pending_airdrop_value.map(|v| v.amount),
        })
    }
}

impl ToProtobuf for PendingAirdropRecord {
    type Protobuf = services::PendingAirdropRecord;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::PendingAirdropRecord {
            pending_airdrop_id: Some(self.pending_airdrop_id.to_protobuf()),
            pending_airdrop_value: self
                .pending_airdrop_value
                .map(|v| hedera_proto::services::PendingAirdropValue { amount: v }),
        }
    }
}
