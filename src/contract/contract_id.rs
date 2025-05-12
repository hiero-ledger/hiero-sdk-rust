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
    PartialEntityId,
    ValidateChecksums,
};
use crate::ethereum::SolidityAddress;
use crate::ledger_id::RefLedgerId;
use crate::{
    Client,
    EntityId,
    Error,
    FromProtobuf,
    ToProtobuf,
};

/// A unique identifier for a smart contract on Hiero.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ContractId {
    /// A non-negative number identifying the shard containing this contract instance.
    pub shard: u64,

    /// A non-negative number identifying the realm within the shard containing this contract instance.
    pub realm: u64,

    /// A non-negative number identifying the entity within the realm containing this contract instance.
    ///
    /// Note: Exactly one of `evm_address` and `num` must exist.
    pub num: u64,

    /// A checksum if the contract ID was read from a user inputted string which inclueded a checksum
    pub checksum: Option<Checksum>,

    /// EVM address identifying the entity within the realm containing this contract instance.
    ///
    /// Note: Exactly one of `evm_address` and `num` must exist.
    pub evm_address: Option<[u8; 20]>,
}

impl ContractId {
    /// Create a `ContractId` from the given shard/realm/num
    #[must_use]
    pub const fn new(shard: u64, realm: u64, num: u64) -> Self {
        Self { shard, realm, num, evm_address: None, checksum: None }
    }

    /// Create a `ContractId` from a `shard.realm.evm_address` set.
    #[must_use]
    pub fn from_evm_address_bytes(shard: u64, realm: u64, evm_address: [u8; 20]) -> Self {
        Self { shard, realm, num: 0, evm_address: Some(evm_address), checksum: None }
    }

    /// Create a `ContractId` from a `shard.realm.evm_address` set.
    ///
    /// # Errors
    /// [`Error::BasicParse`] if `address` is invalid hex, or the wrong length.
    pub fn from_evm_address(shard: u64, realm: u64, address: &str) -> crate::Result<Self> {
        Ok(Self {
            shard,
            realm,
            num: 0,
            evm_address: Some(SolidityAddress::from_str(address)?.to_bytes()),
            checksum: None,
        })
    }

    /// Create a `ContractId` from a solidity address.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if `address` cannot be parsed as a solidity address.
    pub fn from_solidity_address(address: &str) -> crate::Result<Self> {
        let EntityId { shard, realm, num, checksum } = EntityId::from_solidity_address(address)?;

        Ok(Self { shard, realm, num, evm_address: None, checksum })
    }

    /// Create a new `ContractId` from protobuf-encoded `bytes`.
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

    /// Convert `self` into a solidity `address`.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if `self.shard` is larger than `u32::MAX`.
    pub fn to_solidity_address(&self) -> crate::Result<String> {
        if let Some(address) = self.evm_address {
            return Ok(hex::encode(address));
        }

        EntityId { shard: self.shard, realm: self.realm, num: self.num, checksum: None }
            .to_solidity_address()
    }

    /// Convert `self` to a string with a valid checksum.
    ///
    /// # Errors
    /// - [`Error::CannotCreateChecksum`] if self has an `evm_address`.
    pub fn to_string_with_checksum(&self, client: &Client) -> Result<String, Error> {
        if self.evm_address.is_some() {
            Err(Error::CannotCreateChecksum)
        } else {
            Ok(EntityId::to_string_with_checksum(self.to_string(), client))
        }
    }

    /// Validates `self.checksum` (if it exists) for `client`.
    ///
    /// # Errors
    /// - [`Error::BadEntityId`] if there is a checksum, and the checksum is not valid for the client's `ledger_id`.
    pub fn validate_checksum(&self, client: &Client) -> Result<(), Error> {
        if self.evm_address.is_some() {
            Ok(())
        } else {
            EntityId::validate_checksum(self.shard, self.realm, self.num, self.checksum, client)
        }
    }
}

impl ValidateChecksums for ContractId {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        if self.evm_address.is_some() {
            Ok(())
        } else {
            EntityId::validate_checksum_for_ledger_id(
                self.shard,
                self.realm,
                self.num,
                self.checksum,
                ledger_id,
            )
        }
    }
}

impl Debug for ContractId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for ContractId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(address) = &self.evm_address {
            write!(f, "{}.{}.{}", self.shard, self.realm, SolidityAddress::from_ref(address))
        } else {
            write!(f, "{}.{}.{}", self.shard, self.realm, self.num)
        }
    }
}

impl FromProtobuf<services::ContractId> for ContractId {
    fn from_protobuf(pb: services::ContractId) -> crate::Result<Self> {
        let contract = pb_getf!(pb, contract)?;

        let (num, evm_address) = match contract {
            services::contract_id::Contract::ContractNum(it) => (it as u64, None),
            services::contract_id::Contract::EvmAddress(it) => {
                (0, Some(SolidityAddress::try_from(it)?.to_bytes()))
            }
        };

        Ok(Self {
            evm_address,
            num,
            shard: pb.shard_num as u64,
            realm: pb.realm_num as u64,
            checksum: None,
        })
    }
}

impl ToProtobuf for ContractId {
    type Protobuf = services::ContractId;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::ContractId {
            contract: Some(match &self.evm_address {
                Some(address) => services::contract_id::Contract::EvmAddress(address.to_vec()),
                None => services::contract_id::Contract::ContractNum(self.num as i64),
            }),
            realm_num: self.realm as i64,
            shard_num: self.shard as i64,
        }
    }
}

impl From<[u8; 20]> for ContractId {
    fn from(address: [u8; 20]) -> Self {
        Self { shard: 0, realm: 0, num: 0, evm_address: Some(address), checksum: None }
    }
}

impl From<u64> for ContractId {
    fn from(num: u64) -> Self {
        Self { num, shard: 0, realm: 0, evm_address: None, checksum: None }
    }
}

impl FromStr for ContractId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // override the error message for better context.
        let partial = PartialEntityId::from_str(s).map_err(|_| {
            Error::basic_parse(format!(
                "expecting <shard>.<realm>.<num> or <shard>.<realm>.<evm_address>, got `{s}`"
            ))
        })?;

        match partial {
            PartialEntityId::ShortNum(it) => Ok(Self::from(it)),
            PartialEntityId::LongNum(it) => Ok(Self::from(it)),
            PartialEntityId::ShortOther(_) => Err(Error::basic_parse(format!(
                "expecting <shard>.<realm>.<num> or <shard>.<realm>.<evm_address>, got `{s}`"
            ))),
            PartialEntityId::LongOther { shard, realm, last } => {
                let evm_address = Some(SolidityAddress::from_str(last)?.to_bytes());

                Ok(Self { shard, realm, num: 0, evm_address, checksum: None })
            }
        }
    }
}

impl From<EntityId> for ContractId {
    fn from(value: EntityId) -> Self {
        let EntityId { shard, realm, num, checksum } = value;

        Self { shard, realm, num, evm_address: None, checksum }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ContractId;

    #[test]
    fn parse() {
        expect_test::expect!["0.0.5005"]
            .assert_eq(&ContractId::from_str("0.0.5005").unwrap().to_string());
    }

    #[test]
    fn from_solidity_address() {
        expect_test::expect!["0.0.5005"].assert_eq(
            &ContractId::from_solidity_address("000000000000000000000000000000000000138D")
                .unwrap()
                .to_string(),
        );
    }

    #[test]
    fn from_solidity_address_0x() {
        expect_test::expect!["0.0.5005"].assert_eq(
            &ContractId::from_solidity_address("0x000000000000000000000000000000000000138D")
                .unwrap()
                .to_string(),
        );
    }

    #[test]
    fn from_evm_address() {
        expect_test::expect!["1.2.98329e006610472e6b372c080833f6d79ed833cf"].assert_eq(
            &ContractId::from_evm_address(1, 2, "98329e006610472e6B372C080833f6D79ED833cf")
                .unwrap()
                .to_string(),
        )
    }

    #[test]
    fn from_evm_address_0x() {
        expect_test::expect!["1.2.98329e006610472e6b372c080833f6d79ed833cf"].assert_eq(
            &ContractId::from_evm_address(1, 2, "0x98329e006610472e6B372C080833f6D79ED833cf")
                .unwrap()
                .to_string(),
        )
    }

    #[test]
    fn parse_evm_address() {
        expect_test::expect!["1.2.98329e006610472e6b372c080833f6d79ed833cf"].assert_eq(
            &ContractId::from_str("1.2.0x98329e006610472e6B372C080833f6D79ED833cf")
                .unwrap()
                .to_string(),
        )
    }

    #[test]
    fn to_from_bytes() {
        let a = ContractId::from_str("1.2.3").unwrap();
        assert_eq!(ContractId::from_bytes(&a.to_bytes()).unwrap(), a);
        let b = ContractId::from_evm_address(1, 2, "0x98329e006610472e6B372C080833f6D79ED833cf")
            .unwrap();
        assert_eq!(ContractId::from_bytes(&b.to_bytes()).unwrap(), b);
    }

    #[test]
    fn to_solidity_address() {
        expect_test::expect!["000000000000000000000000000000000000138d"].assert_eq(
            &ContractId { shard: 0, realm: 0, num: 5005, checksum: None, evm_address: None }
                .to_solidity_address()
                .unwrap(),
        )
    }

    #[test]
    fn to_solidity_address_2() {
        expect_test::expect!["98329e006610472e6b372c080833f6d79ed833cf"].assert_eq(
            &ContractId::from_evm_address(1, 2, "0x98329e006610472e6B372C080833f6D79ED833cf")
                .unwrap()
                .to_solidity_address()
                .unwrap(),
        )
    }
}
