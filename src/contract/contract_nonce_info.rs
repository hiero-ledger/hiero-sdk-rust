// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::ContractId;

/// Info about a contract account's nonce value.
/// The nonce for a contract is only incremented when that contract creates another contract.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ContractNonceInfo {
    /// The contract's ID.
    pub contract_id: ContractId,
    /// The contract's nonce.
    pub nonce: u64,
}

impl ContractNonceInfo {
    /// Create a new `ContractNonceInfo` from protobuf-encoded `bytes`.
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

impl FromProtobuf<services::ContractNonceInfo> for ContractNonceInfo {
    fn from_protobuf(pb: services::ContractNonceInfo) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            contract_id: ContractId::from_protobuf(pb_getf!(pb, contract_id)?)?,
            nonce: pb.nonce as u64,
        })
    }
}

impl ToProtobuf for ContractNonceInfo {
    type Protobuf = services::ContractNonceInfo;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            contract_id: Some(self.contract_id.to_protobuf()),
            nonce: self.nonce as i64,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::ContractNonceInfo;

    const INFO: services::ContractNonceInfo = services::ContractNonceInfo {
        contract_id: Some(services::ContractId {
            shard_num: 0,
            realm_num: 0,
            contract: Some(services::contract_id::Contract::ContractNum(2)),
        }),
        nonce: 2,
    };

    #[test]
    fn from_protobuf() {
        expect![[r#"
            ContractNonceInfo {
                contract_id: "0.0.2",
                nonce: 2,
            }
        "#]]
        .assert_debug_eq(&ContractNonceInfo::from_protobuf(INFO).unwrap());
    }

    #[test]
    fn to_protobuf() {
        expect![[r#"
            ContractNonceInfo {
                contract_id: Some(
                    ContractId {
                        shard_num: 0,
                        realm_num: 0,
                        contract: Some(
                            ContractNum(
                                2,
                            ),
                        ),
                    },
                ),
                nonce: 2,
            }
        "#]]
        .assert_debug_eq(&ContractNonceInfo::from_protobuf(INFO).unwrap().to_protobuf());
    }

    #[test]
    fn from_bytes() {
        expect![[r#"
            ContractNonceInfo {
                contract_id: Some(
                    ContractId {
                        shard_num: 0,
                        realm_num: 0,
                        contract: Some(
                            ContractNum(
                                2,
                            ),
                        ),
                    },
                ),
                nonce: 2,
            }
        "#]]
        .assert_debug_eq(
            &ContractNonceInfo::from_bytes(&prost::Message::encode_to_vec(&INFO))
                .unwrap()
                .to_protobuf(),
        );
    }
}
