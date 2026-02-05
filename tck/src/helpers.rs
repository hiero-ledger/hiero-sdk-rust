use std::collections::HashMap;
use std::str::FromStr;

use hex::ToHex;
use hiero_sdk::{
    AccountId,
    Client,
    Hbar,
    Key,
    KeyList,
    PrivateKey,
    PublicKey,
    Transaction,
    TransactionId,
};
use hiero_sdk_proto::services::key::Key as ProtoKeyEnum;
use hiero_sdk_proto::services::{
    Key as ProtoKey,
    KeyList as ProtoKeyList,
    ThresholdKey as ProtoThresholdKey,
};
use jsonrpsee::types::error::INVALID_PARAMS_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use prost::Message;
use serde_json::Value;
use time::Duration;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum KeyType {
    Ed25519PrivateKeyType,
    Ed25519PublicKeyType,
    EcdsaSecp256k1PrivateKeyType,
    EcdsaSecp256k1PublicKeyType,
    KeyListType,
    ThresholdKeyType,
    EvmAddressType,
}

impl FromStr for KeyType {
    type Err = ErrorObjectOwned;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ed25519PrivateKey" => Ok(KeyType::Ed25519PrivateKeyType),
            "ed25519PublicKey" => Ok(KeyType::Ed25519PublicKeyType),
            "ecdsaSecp256k1PrivateKey" => Ok(KeyType::EcdsaSecp256k1PrivateKeyType),
            "ecdsaSecp256k1PublicKey" => Ok(KeyType::EcdsaSecp256k1PublicKeyType),
            "keyList" => Ok(KeyType::KeyListType),
            "thresholdKey" => Ok(KeyType::ThresholdKeyType),
            "evmAddress" => Ok(KeyType::EvmAddressType),
            _ => Err(ErrorObject::borrowed(-32603, "generateKey: type is NOT a valid value", None)),
        }
    }
}

pub(crate) fn fill_common_transaction_params<D: hiero_sdk::ValidateChecksums>(
    transaction: &mut Transaction<D>,
    common_transaction_params: &HashMap<String, Value>,
    client: &Client,
) {
    if let Some(transaction_id) = common_transaction_params.get("transactionId") {
        match transaction_id {
            Value::String(transaction_id) => {
                transaction
                    .transaction_id(TransactionId::from_str(transaction_id.as_str()).unwrap());
            }
            _ => {}
        }
    }

    if let Some(node_id) = common_transaction_params.get("nodeId") {
        match node_id {
            Value::String(node_id) => {
                transaction.node_account_ids([AccountId::from_str(&node_id.as_str()).unwrap()]);
            }
            _ => {}
        }
    }

    if let Some(max_fee) = common_transaction_params.get("maxTransactionFee") {
        let fee_tinybars = match max_fee {
            Value::String(s) => s.as_str().parse::<i64>().ok(),
            Value::Number(n) => n.as_i64(),
            _ => None,
        };

        if let Some(tinybars) = fee_tinybars {
            transaction.max_transaction_fee(Hbar::from_tinybars(tinybars));
        }
    }

    if let Some(transaction_valid_duration) =
        common_transaction_params.get("transactionValidDuration")
    {
        match transaction_valid_duration {
            Value::String(transaction_valid_duration) => {
                transaction.transaction_valid_duration(Duration::seconds(
                    transaction_valid_duration.as_str().parse::<i64>().unwrap(),
                ));
            }
            _ => {}
        }
    }

    if let Some(memo) = common_transaction_params.get("memo") {
        match memo {
            Value::String(memo) => {
                transaction.transaction_memo(memo.as_str());
            }
            _ => {}
        }
    }

    if let Some(signers) = common_transaction_params.get("signers") {
        if let Err(err) = transaction.freeze_with(client) {
            panic!("Failed to freeze transaction: {:?}", err);
        }

        // Get operator public key for comparison
        let operator_public_key = client.get_operator_public_key();

        match signers {
            Value::Array(signers) => {
                for signer in signers {
                    if let Value::String(signer_str) = signer {
                        // Skip if this signer's public key matches the operator key
                        if let Some(ref op_key) = operator_public_key {
                            if let Ok(private_key) = PrivateKey::from_str(signer_str.as_str()) {
                                if &private_key.public_key() == op_key {
                                    continue;
                                }
                            }
                        }
                        transaction.sign(PrivateKey::from_str(signer_str.as_str()).unwrap());
                    }
                }
            }
            _ => {}
        }
    }
}
pub(crate) fn generate_key_helper(
    _type: String,
    from_key: Option<String>,
    threshold: Option<i32>,
    keys: Option<Value>,
    private_keys: &mut Vec<Value>,
    is_list: bool,
) -> Result<String, ErrorObjectOwned> {
    // Check the key type
    let key_type = KeyType::from_str(&_type)?;

    if from_key.is_some()
        && key_type != KeyType::Ed25519PublicKeyType
        && key_type != KeyType::EcdsaSecp256k1PublicKeyType
        && key_type != KeyType::EvmAddressType
    {
        return Err(ErrorObject::borrowed(INVALID_PARAMS_CODE, "generateKey: fromKey MUST NOT be provided for types other than ed25519PublicKey, ecdsaSecp256k1PublicKey, or evmAddress.", None));
    }

    if threshold.is_some() && key_type != KeyType::ThresholdKeyType {
        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: threshold MUST NOT be provided for types other than thresholdKey.",
            None,
        ));
    } else if threshold.is_none() && key_type == KeyType::ThresholdKeyType {
        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: threshold MUST be provided for thresholdKey types.",
            None,
        ));
    };

    if keys.is_some() && key_type != KeyType::ThresholdKeyType && key_type != KeyType::KeyListType {
        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: keys MUST NOT be provided for types other than keyList or thresholdKey.",
            None,
        ));
    } else if keys.is_none()
        && (key_type == KeyType::ThresholdKeyType || key_type == KeyType::KeyListType)
    {
        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: keys MUST be provided for keyList and thresholdKey types.",
            None,
        ));
    };

    match key_type {
        KeyType::Ed25519PrivateKeyType | KeyType::EcdsaSecp256k1PrivateKeyType => {
            let key = if key_type == KeyType::Ed25519PublicKeyType {
                PrivateKey::generate_ed25519().to_string_der()
            } else {
                PrivateKey::generate_ecdsa().to_string_der()
            };

            if is_list {
                private_keys.push(Value::String(key.clone()));
            }

            return Ok(key);
        }
        KeyType::Ed25519PublicKeyType | KeyType::EcdsaSecp256k1PublicKeyType => {
            if let Some(from_key) = from_key {
                return PrivateKey::from_str_der(&from_key)
                    .map(|key| key.public_key().to_string_der())
                    .map_err(|_| {
                        ErrorObject::borrowed(
                            INVALID_PARAMS_CODE,
                            "generateKey: could not produce {key_type:?}",
                            None,
                        )
                    });
            };

            let key = if key_type == KeyType::Ed25519PublicKeyType {
                PrivateKey::generate_ed25519()
            } else {
                PrivateKey::generate_ecdsa()
            };

            if is_list {
                private_keys.push(Value::String(key.to_string_der()));
            }

            return Ok(key.public_key().to_string_der());
        }
        KeyType::KeyListType | KeyType::ThresholdKeyType => {
            let mut key_list = KeyList::new();

            if let Value::Array(key_array) = keys.unwrap() {
                for key in key_array {
                    let generate_key = &generate_key_helper(
                        key["type"].as_str().unwrap().to_string(),
                        None,
                        None,
                        key.get("keys").map(|value| value.clone()),
                        private_keys,
                        true,
                    )?;

                    let get_key = get_hedera_key(&generate_key)?;

                    key_list.keys.push(get_key);
                }
            }

            if KeyType::from_str(&_type)? == KeyType::ThresholdKeyType {
                key_list.threshold = Some(threshold.unwrap() as u32);
            }

            return Ok(Key::KeyList(key_list).to_bytes().encode_hex());
        }
        KeyType::EvmAddressType => {
            if from_key.is_none() {
                return Ok(PrivateKey::generate_ecdsa()
                    .public_key()
                    .to_evm_address()
                    .unwrap()
                    .to_string());
            }

            let private_key = PrivateKey::from_str_ecdsa(&from_key.clone().unwrap());

            match private_key {
                Ok(key) => {
                    return Ok(key.public_key().to_evm_address().unwrap().to_string());
                }
                Err(_) => {
                    let private_key = PublicKey::from_str_ecdsa(&from_key.unwrap());

                    match private_key {
                        Ok(key) => {
                            return Ok(key.to_evm_address().unwrap().to_string());
                        }
                        Err(_) => {
                            return Err(ErrorObject::borrowed(INVALID_PARAMS_CODE, "generateKey: fromKey for evmAddress MUST be an ECDSAsecp256k1 private or public key.", None));
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn get_hedera_key(key: &str) -> Result<Key, ErrorObjectOwned> {
    match PrivateKey::from_str_der(key).map(|pk| Key::Single(pk.public_key())) {
        Ok(key) => Ok(key),
        Err(_) => match PublicKey::from_str_der(key).map(Key::Single) {
            Ok(key) => Ok(key),
            Err(_) => {
                // try plain ed25519 pub key string
                if let Ok(public_key) = PublicKey::from_str_ed25519(key) {
                    return Ok(public_key.into());
                }

                // try hex-encoded serialized services::Key (used for keyList/threshold outputs)
                if let Ok(bytes) = hex::decode(key) {
                    if let Ok(pb_key) = ProtoKey::decode(bytes.as_slice()) {
                        if let Some(k) = key_from_proto(pb_key) {
                            return Ok(k);
                        }
                    }
                }

                Err(ErrorObject::borrowed(-32603, "generateKey: fromKey is invalid.", None))
            }
        },
    }
}

fn key_from_proto(pb_key: ProtoKey) -> Option<Key> {
    match pb_key.key? {
        ProtoKeyEnum::Ed25519(bytes) => PublicKey::from_bytes_ed25519(&bytes).ok().map(Key::Single),
        ProtoKeyEnum::EcdsaSecp256k1(bytes) => {
            PublicKey::from_bytes_ecdsa(&bytes).ok().map(Key::Single)
        }
        ProtoKeyEnum::KeyList(list) => keylist_from_proto(list).map(Key::KeyList),
        ProtoKeyEnum::ThresholdKey(th) => keylist_from_threshold_proto(th).map(Key::KeyList),
        _ => None,
    }
}

fn keylist_from_proto(pb_list: ProtoKeyList) -> Option<KeyList> {
    let mut keys = Vec::new();
    for k in pb_list.keys {
        if let Some(key) = key_from_proto(ProtoKey { key: k.key }) {
            keys.push(key);
        } else {
            return None;
        }
    }
    Some(KeyList::from(keys))
}

fn keylist_from_threshold_proto(pb_th: ProtoThresholdKey) -> Option<KeyList> {
    let list = pb_th.keys?;
    let mut key_list = keylist_from_proto(list)?;
    key_list.threshold = Some(pb_th.threshold as u32);
    Some(key_list)
}

/// Converts a `Key` to its DER string representation.
/// For single keys, returns the DER-encoded hex string.
/// For key lists, returns the hex-encoded protobuf representation.
pub(crate) fn key_to_der_string(key: &Key) -> String {
    match key {
        Key::Single(public_key) => public_key.to_string_der(),
        Key::KeyList(key_list) => Key::KeyList(key_list.clone()).to_bytes().encode_hex(),
        Key::ContractId(contract_id) => contract_id.to_string(),
        Key::DelegateContractId(delegate_id) => delegate_id.to_string(),
        _ => format!("{:?}", key), // Fallback for any future key types
    }
}
