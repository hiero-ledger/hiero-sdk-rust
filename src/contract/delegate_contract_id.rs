// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::str::FromStr;

use hedera_proto::services;

use crate::entity_id::{
    Checksum,
    PartialEntityId,
};
use crate::ethereum::SolidityAddress;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::{
    EntityId,
    Error,
};

/// A unique identifier for a smart contract on Hiero.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct DelegateContractId {
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

impl DelegateContractId {
    /// Create a `DelegateContractId` from the given shard/realm/num
    #[must_use]
    pub fn new(shard: u64, realm: u64, num: u64) -> Self {
        Self { shard, realm, num, evm_address: None, checksum: None }
    }

    /// Create a `DelegateContractId` from a solidity address.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if `address` cannot be parsed as a solidity address.
    pub fn from_solidity_address(address: &str) -> crate::Result<Self> {
        let EntityId { shard, realm, num, checksum } = EntityId::from_solidity_address(address)?;

        Ok(Self { shard, realm, num, evm_address: None, checksum })
    }
}

impl fmt::Debug for DelegateContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl fmt::Display for DelegateContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(address) = &self.evm_address {
            write!(f, "{}.{}.{}", self.shard, self.realm, SolidityAddress::from_ref(address))
        } else {
            write!(f, "{}.{}.{}", self.shard, self.realm, self.num)
        }
    }
}

impl FromStr for DelegateContractId {
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

impl From<[u8; 20]> for DelegateContractId {
    fn from(address: [u8; 20]) -> Self {
        Self { shard: 0, realm: 0, num: 0, evm_address: Some(address), checksum: None }
    }
}

impl From<u64> for DelegateContractId {
    fn from(num: u64) -> Self {
        Self::new(0, 0, num)
    }
}

impl From<EntityId> for DelegateContractId {
    fn from(value: EntityId) -> Self {
        let EntityId { shard, realm, num, checksum } = value;

        Self { shard, realm, num, evm_address: None, checksum }
    }
}

impl FromProtobuf<services::ContractId> for DelegateContractId {
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

impl ToProtobuf for DelegateContractId {
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use expect_test::expect;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::DelegateContractId;

    #[test]
    fn from_string() {
        expect!["0.0.5005"]
            .assert_eq(&DelegateContractId::from_str("0.0.5005").unwrap().to_string());
    }

    #[test]
    fn from_solidity_address() {
        expect!["0.0.5005"].assert_eq(
            &DelegateContractId::from_solidity_address("000000000000000000000000000000000000138D")
                .unwrap()
                .to_string(),
        );
    }

    #[test]
    fn from_solidity_address_with_0x() {
        expect!["0.0.5005"].assert_eq(
            &DelegateContractId::from_solidity_address(
                "0x000000000000000000000000000000000000138D",
            )
            .unwrap()
            .to_string(),
        );
    }

    #[test]
    fn from_bytes() {
        expect!["0.0.5005"].assert_eq(
            &DelegateContractId::from_bytes(&DelegateContractId::new(0, 0, 5005).to_bytes())
                .unwrap()
                .to_string(),
        );
    }
}
