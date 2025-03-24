// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;

use crate::contract::DelegateContractId;
use crate::{
    ContractId,
    Error,
    FromProtobuf,
    KeyList,
    PublicKey,
    ToProtobuf,
};

/// Any method that can be used to authorize an operation on Hedera.
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
#[non_exhaustive]
pub enum Key {
    // todo(sr): not happy with any of these (fix before merge)
    /// A single public key.
    Single(PublicKey),

    /// A contract ID.
    ContractId(ContractId),

    /// A delegatable contract ID.
    DelegateContractId(DelegateContractId),

    /// A key list.
    KeyList(KeyList),
}

impl Key {
    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl ToProtobuf for Key {
    type Protobuf = services::Key;

    fn to_protobuf(&self) -> Self::Protobuf {
        use services::key::Key::*;

        services::Key {
            key: Some(match self {
                Self::Single(key) => {
                    let bytes = key.to_bytes_raw();

                    match key.kind() {
                        crate::key::KeyKind::Ed25519 => Ed25519(bytes),
                        crate::key::KeyKind::Ecdsa => EcdsaSecp256k1(bytes),
                    }
                }

                Self::ContractId(id) => ContractId(id.to_protobuf()),
                Self::DelegateContractId(id) => DelegatableContractId(id.to_protobuf()),
                // `KeyList`s are special and can be both a key list and a threshold key.
                Self::KeyList(key) => key.to_protobuf_key(),
            }),
        }
    }
}

impl From<PublicKey> for Key {
    fn from(key: PublicKey) -> Self {
        Self::Single(key)
    }
}

impl From<ContractId> for Key {
    fn from(id: ContractId) -> Self {
        Self::ContractId(id)
    }
}

impl From<KeyList> for Key {
    fn from(value: KeyList) -> Self {
        Self::KeyList(value)
    }
}

impl FromProtobuf<services::Key> for Key {
    fn from_protobuf(pb: services::Key) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use services::key::Key::*;

        match pb.key {
            Some(Ed25519(bytes)) => Ok(Self::Single(PublicKey::from_bytes_ed25519(&bytes)?)),
            Some(ContractId(id)) => Ok(Self::ContractId(crate::ContractId::from_protobuf(id)?)),
            Some(DelegatableContractId(id)) => {
                Ok(Self::DelegateContractId(crate::DelegateContractId::from_protobuf(id)?))
            }
            Some(Rsa3072(_)) => {
                Err(Error::from_protobuf("unexpected unsupported RSA-3072 key in Key"))
            }
            Some(Ecdsa384(_)) => {
                Err(Error::from_protobuf("unexpected unsupported ECDSA-384 key in Key"))
            }
            Some(ThresholdKey(it)) => Ok(Self::KeyList(crate::KeyList::from_protobuf(it)?)),
            Some(KeyList(it)) => Ok(Self::KeyList(crate::KeyList::from_protobuf(it)?)),
            Some(EcdsaSecp256k1(bytes)) => Ok(Self::Single(PublicKey::from_bytes_ecdsa(&bytes)?)),
            None => Err(Error::from_protobuf("unexpected empty key in Key")),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use hedera_proto::services;
    use hex_literal::hex;

    use crate::protobuf::FromProtobuf;
    use crate::{
        Key,
        PublicKey,
    };

    #[test]
    fn from_proto_key_ed25519() {
        const KEY_BYTES: [u8; 32] =
            hex!("0011223344556677889900112233445566778899001122334455667788990011");

        let key = services::Key { key: Some(services::key::Key::Ed25519(KEY_BYTES.to_vec())) };

        let key = PublicKey::from_protobuf(key).unwrap();

        assert_matches!(key.kind(), crate::key::KeyKind::Ed25519);

        assert_eq!(key.to_bytes_raw(), KEY_BYTES);
    }

    #[test]
    fn from_proto_key_ecdsa() {
        const KEY_BYTES: [u8; 35] =
            hex!("3a21034e0441201f2bf9c7d9873c2a9dc3fd451f64b7c05e17e4d781d916e3a11dfd99");

        let key = PublicKey::from_alias_bytes(&KEY_BYTES).unwrap().unwrap();

        assert_matches!(key.kind(), crate::key::KeyKind::Ecdsa);

        assert_eq!(Key::from(key).to_bytes(), KEY_BYTES);
    }

    #[test]
    fn from_proto_key_key_list() {
        const KEY_BYTES: [[u8; 32]; 2] = [
            hex!("0011223344556677889900112233445566778899001122334455667788990011"),
            hex!("aa11223344556677889900112233445566778899001122334455667788990011"),
        ];

        let key_list_pb = services::KeyList {
            keys: KEY_BYTES
                .iter()
                .map(|it| services::Key { key: Some(services::key::Key::Ed25519(it.to_vec())) })
                .collect(),
        };

        let key_pb = services::Key { key: Some(services::key::Key::KeyList(key_list_pb.clone())) };

        let key = Key::from_protobuf(key_pb).unwrap();

        let key_list = assert_matches!(key, Key::KeyList(it) => it);

        assert_eq!(key_list.len(), KEY_BYTES.len());

        let reencoded =
            assert_matches!(key_list.to_protobuf_key(), services::key::Key::KeyList(key) => key);

        assert_eq!(reencoded, key_list_pb);
    }

    #[test]
    fn from_proto_key_threshold_key() {
        const KEY_BYTES: [[u8; 32]; 2] = [
            hex!("0011223344556677889900112233445566778899001122334455667788990011"),
            hex!("aa11223344556677889900112233445566778899001122334455667788990011"),
        ];

        let key_list_pb = services::KeyList {
            keys: KEY_BYTES
                .iter()
                .map(|it| services::Key { key: Some(services::key::Key::Ed25519(it.to_vec())) })
                .collect(),
        };

        let threshold_key_pb = services::ThresholdKey { threshold: 1, keys: Some(key_list_pb) };

        let key_pb =
            services::Key { key: Some(services::key::Key::ThresholdKey(threshold_key_pb.clone())) };

        let key = Key::from_protobuf(key_pb).unwrap();

        let threshold_key = assert_matches!(key, Key::KeyList(it) => it);

        assert_eq!(threshold_key.len(), KEY_BYTES.len());

        let reencoded = assert_matches!(threshold_key.to_protobuf_key(), services::key::Key::ThresholdKey(key) => key);

        assert_eq!(reencoded, threshold_key_pb);
    }

    #[test]
    fn unsupported_key_fails() {
        let key = services::Key { key: Some(services::key::Key::Rsa3072(Vec::from([0, 1, 2]))) };

        assert_matches!(Key::from_protobuf(key), Err(crate::Error::FromProtobuf(_)));
    }
}
