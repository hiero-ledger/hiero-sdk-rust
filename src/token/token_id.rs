// SPDX-License-Identifier: Apache-2.0

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;

use hedera_proto::services;

use crate::entity_id::{
    Checksum,
    ValidateChecksums,
};
use crate::{
    Client,
    EntityId,
    Error,
    FromProtobuf,
    NftId,
    ToProtobuf,
};

/// The unique identifier for a token on Hiero.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct TokenId {
    /// A non-negative number identifying the shard containing this token.
    pub shard: u64,

    /// A non-negative number identifying the realm within the shard containing this token.
    pub realm: u64,

    /// A non-negative number identifying the entity within the realm containing this token.
    pub num: u64,

    /// A checksum if the token ID was read from a user inputted string which inclueded a checksum
    pub checksum: Option<Checksum>,
}

impl TokenId {
    /// Create a `TokenId` from the given `shard`, `realm`, and `num`.
    #[must_use]
    pub const fn new(shard: u64, realm: u64, num: u64) -> Self {
        Self { shard, realm, num, checksum: None }
    }

    /// Create a new `TokenId` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Create a `TokenId` from a solidity address.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if `address` cannot be parsed as a solidity address.
    pub fn from_solidity_address(address: &str) -> crate::Result<Self> {
        let EntityId { shard, realm, num, checksum } = EntityId::from_solidity_address(address)?;

        Ok(Self { shard, realm, num, checksum })
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }

    /// Convert `self` into a solidity `address`.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if `self.shard` is larger than `u32::MAX`.
    pub fn to_solidity_address(&self) -> crate::Result<String> {
        EntityId { shard: self.shard, realm: self.realm, num: self.num, checksum: None }
            .to_solidity_address()
    }

    /// Convert `self` to a string with a valid checksum.
    #[must_use]
    pub fn to_string_with_checksum(&self, client: &Client) -> String {
        EntityId::to_string_with_checksum(self.to_string(), client)
    }

    /// Validates `self.checksum` (if it exists) for `client`.
    ///
    /// # Errors
    /// - [`Error::BadEntityId`] if there is a checksum, and the checksum is not valid for the client's `ledger_id`.
    pub fn validate_checksum(&self, client: &Client) -> crate::Result<()> {
        EntityId::validate_checksum(self.shard, self.realm, self.num, self.checksum, client)
    }

    /// Create an NFT ID
    #[must_use]
    pub fn nft(&self, serial: u64) -> NftId {
        NftId { token_id: *self, serial }
    }
}

impl ValidateChecksums for TokenId {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        EntityId::validate_checksum_for_ledger_id(
            self.shard,
            self.realm,
            self.num,
            self.checksum,
            ledger_id,
        )
    }
}

impl Debug for TokenId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for TokenId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.shard, self.realm, self.num)
    }
}

impl FromProtobuf<services::TokenId> for TokenId {
    fn from_protobuf(pb: services::TokenId) -> crate::Result<Self> {
        Ok(Self {
            num: pb.token_num as u64,
            shard: pb.shard_num as u64,
            realm: pb.realm_num as u64,
            checksum: None,
        })
    }
}

impl ToProtobuf for TokenId {
    type Protobuf = services::TokenId;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::TokenId {
            token_num: self.num as i64,
            realm_num: self.realm as i64,
            shard_num: self.shard as i64,
        }
    }
}

impl From<u64> for TokenId {
    fn from(num: u64) -> Self {
        Self { num, shard: 0, realm: 0, checksum: None }
    }
}

impl FromStr for TokenId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EntityId::from_str(s).map(Self::from)
    }
}

impl From<EntityId> for TokenId {
    fn from(value: EntityId) -> Self {
        let EntityId { shard, realm, num, checksum } = value;

        Self { shard, realm, num, checksum }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use expect_test::expect;

    use crate::TokenId;

    #[test]
    fn parse() {
        expect!["0.0.5005"].assert_eq(&TokenId::from_str("0.0.5005").unwrap().to_string());
    }

    #[test]
    fn from_bytes() {
        expect!["0.0.5005"].assert_eq(
            &TokenId::from_bytes(&TokenId::new(0, 0, 5005).to_bytes()).unwrap().to_string(),
        );
    }

    #[test]
    fn from_solidity_address() {
        expect!["0.0.5005"].assert_eq(
            &TokenId::from_solidity_address("000000000000000000000000000000000000138D")
                .unwrap()
                .to_string(),
        );
    }

    #[test]
    fn to_solidity_address() {
        expect!["000000000000000000000000000000000000138d"]
            .assert_eq(&TokenId::new(0, 0, 5005).to_solidity_address().unwrap());
    }
}
