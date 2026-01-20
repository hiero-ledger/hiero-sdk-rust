use std::collections::HashMap;
use std::str::FromStr;

use hex::ToHex;
use hiero_sdk::{
    AccountId,
    Hbar,
    Key,
    KeyList,
    PrivateKey,
    PublicKey,
    Transaction,
    TransactionId,
};
use hiero_sdk_proto::services;
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

pub(crate) fn fill_common_transaction_params<D>(
    transaction: &mut Transaction<D>,
    common_transaction_params: &HashMap<String, Value>,
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
        match max_fee {
            Value::String(max_fee) => {
                transaction.max_transaction_fee(Hbar::from_tinybars(
                    max_fee.as_str().parse::<i64>().unwrap(),
                ));
            }
            _ => {}
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
            let key = if key_type == KeyType::Ed25519PrivateKeyType {
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
                // Try parsing as DER first, then try other formats
                let private_key = PrivateKey::from_str_der(&from_key)
                    .or_else(|_| {
                        // Try parsing as Ed25519 raw key
                        if key_type == KeyType::Ed25519PublicKeyType {
                            PrivateKey::from_str_ed25519(&from_key)
                        } else {
                            PrivateKey::from_str_ecdsa(&from_key)
                        }
                    })
                    .map_err(|e| {
                        ErrorObject::owned(
                            INVALID_PARAMS_CODE,
                            format!("generateKey: fromKey is invalid: {}", e),
                            None::<()>,
                        )
                    })?;

                return Ok(private_key.public_key().to_string_der());
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
                    let key_from_key = key.get("fromKey").and_then(|v| {
                        // Check if fromKey is an index (number or numeric string) into private_keys
                        if let Some(idx) = v.as_u64() {
                            private_keys
                                .get(idx as usize)
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        } else if let Some(idx_str) = v.as_str() {
                            // Try parsing as index first
                            if let Ok(idx) = idx_str.parse::<usize>() {
                                private_keys
                                    .get(idx)
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                            } else {
                                // Not an index, treat as direct key string
                                Some(idx_str.to_string())
                            }
                        } else {
                            None
                        }
                    });

                    let key_type_str = key["type"].as_str().unwrap().to_string();
                    let is_nested_key_list =
                        key_type_str == "keyList" || key_type_str == "thresholdKey";

                    let generate_key = &generate_key_helper(
                        key_type_str,
                        key_from_key,
                        None,
                        key.get("keys").map(|value| value.clone()),
                        private_keys,
                        true,
                    )?;

                    let get_key = if is_nested_key_list {
                        // For nested KeyLists, decode the hex-encoded protobuf bytes directly
                        let bytes = hex::decode(generate_key).map_err(|e| {
                            ErrorObject::owned(
                                INVALID_PARAMS_CODE,
                                format!("generateKey: failed to decode hex key: {}", e),
                                None::<()>,
                            )
                        })?;
                        // Decode protobuf bytes using prost
                        let proto_key = services::Key::decode(bytes.as_slice()).map_err(|e| {
                            ErrorObject::owned(
                                INVALID_PARAMS_CODE,
                                format!("generateKey: failed to decode protobuf key: {}", e),
                                None::<()>,
                            )
                        })?;
                        // Manually convert from protobuf Key to hiero_sdk::Key
                        // (replicating FromProtobuf logic since trait is private)
                        use services::key::Key::*;
                        match proto_key.key {
                            Some(Ed25519(key_bytes)) => Key::Single(
                                PublicKey::from_bytes_ed25519(&key_bytes).map_err(|e| {
                                    ErrorObject::owned(
                                        INVALID_PARAMS_CODE,
                                        format!("generateKey: failed to parse Ed25519 key: {}", e),
                                        None::<()>,
                                    )
                                })?,
                            ),
                            Some(EcdsaSecp256k1(key_bytes)) => Key::Single(
                                PublicKey::from_bytes_ecdsa(&key_bytes).map_err(|e| {
                                    ErrorObject::owned(
                                        INVALID_PARAMS_CODE,
                                        format!("generateKey: failed to parse ECDSA key: {}", e),
                                        None::<()>,
                                    )
                                })?,
                            ),
                            Some(KeyList(key_list_proto)) => {
                                // For KeyList, we need to recursively process the keys
                                let mut key_list = hiero_sdk::KeyList::new();
                                for proto_key_inner in key_list_proto.keys {
                                    // Decode each key in the list from protobuf
                                    use services::key::Key::*;
                                    let inner_key = match proto_key_inner.key {
                                        Some(Ed25519(key_bytes)) => {
                                            Key::Single(PublicKey::from_bytes_ed25519(&key_bytes).map_err(|e| {
                                                ErrorObject::owned(
                                                    INVALID_PARAMS_CODE,
                                                    format!("generateKey: failed to parse Ed25519 key: {}", e),
                                                    None::<()>,
                                                )
                                            })?)
                                        }
                                        Some(EcdsaSecp256k1(key_bytes)) => {
                                            Key::Single(PublicKey::from_bytes_ecdsa(&key_bytes).map_err(|e| {
                                                ErrorObject::owned(
                                                    INVALID_PARAMS_CODE,
                                                    format!("generateKey: failed to parse ECDSA key: {}", e),
                                                    None::<()>,
                                                )
                                            })?)
                                        }
                                        Some(KeyList(_)) | Some(ThresholdKey(_)) => {
                                            // Nested KeyList - this case should be handled at a higher level
                                            // For now, return an error
                                            return Err(ErrorObject::owned(
                                                INVALID_PARAMS_CODE,
                                                "generateKey: deeply nested KeyLists require special handling",
                                                None::<()>,
                                            ));
                                        }
                                        _ => {
                                            return Err(ErrorObject::owned(
                                                INVALID_PARAMS_CODE,
                                                "generateKey: unsupported key type in KeyList",
                                                None::<()>,
                                            ));
                                        }
                                    };
                                    key_list.keys.push(inner_key);
                                }
                                Key::KeyList(key_list)
                            }
                            Some(ThresholdKey(services::ThresholdKey {
                                keys: Some(key_list_proto),
                                threshold: threshold_val,
                                ..
                            })) => {
                                // For ThresholdKey, process similarly but set threshold
                                let mut key_list = hiero_sdk::KeyList::new();
                                for proto_key_inner in key_list_proto.keys {
                                    use services::key::Key::*;
                                    let inner_key = match proto_key_inner.key {
                                        Some(Ed25519(key_bytes)) => {
                                            Key::Single(PublicKey::from_bytes_ed25519(&key_bytes).map_err(|e| {
                                                ErrorObject::owned(
                                                    INVALID_PARAMS_CODE,
                                                    format!("generateKey: failed to parse Ed25519 key: {}", e),
                                                    None::<()>,
                                                )
                                            })?)
                                        }
                                        Some(EcdsaSecp256k1(key_bytes)) => {
                                            Key::Single(PublicKey::from_bytes_ecdsa(&key_bytes).map_err(|e| {
                                                ErrorObject::owned(
                                                    INVALID_PARAMS_CODE,
                                                    format!("generateKey: failed to parse ECDSA key: {}", e),
                                                    None::<()>,
                                                )
                                            })?)
                                        }
                                        Some(KeyList(_)) | Some(ThresholdKey(_)) => {
                                            return Err(ErrorObject::owned(
                                                INVALID_PARAMS_CODE,
                                                "generateKey: deeply nested KeyLists require special handling",
                                                None::<()>,
                                            ));
                                        }
                                        _ => {
                                            return Err(ErrorObject::owned(
                                                INVALID_PARAMS_CODE,
                                                "generateKey: unsupported key type in ThresholdKey",
                                                None::<()>,
                                            ));
                                        }
                                    };
                                    key_list.keys.push(inner_key);
                                }
                                // Set threshold (convert from protobuf u32 to Option<u32>)
                                key_list.threshold = Some(threshold_val);
                                Key::KeyList(key_list)
                            }
                            _ => {
                                return Err(ErrorObject::owned(
                                    INVALID_PARAMS_CODE,
                                    "generateKey: unsupported key type in nested KeyList",
                                    None::<()>,
                                ));
                            }
                        }
                    } else {
                        // For regular keys, use get_hedera_key to parse DER-encoded strings
                        get_hedera_key(&generate_key)?
                    };

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
    // First, try parsing as DER-encoded private key
    if let Ok(pk) = PrivateKey::from_str_der(key) {
        return Ok(Key::Single(pk.public_key()));
    }

    // Try parsing as DER-encoded public key
    if let Ok(pk) = PublicKey::from_str_der(key) {
        return Ok(Key::Single(pk));
    }

    // Try parsing as hex-encoded protobuf Key (for KeyLists)
    if let Ok(bytes) = hex::decode(key) {
        if let Ok(proto_key) = services::Key::decode(bytes.as_slice()) {
            return parse_proto_key_to_hedera_key(proto_key);
        }
    }

    // Finally, try as raw ed25519 public key
    let public_key = PublicKey::from_str_ed25519(key).map_err(|e| {
        ErrorObject::owned(
            -32603,
            format!("generateKey: fromKey is invalid. Key: '{}', Error: {}", key, e),
            None::<()>,
        )
    })?;

    Ok(public_key.into())
}

/// Helper function to convert protobuf Key to hiero_sdk::Key
fn parse_proto_key_to_hedera_key(proto_key: services::Key) -> Result<Key, ErrorObjectOwned> {
    use services::key::Key::*;

    match proto_key.key {
        Some(Ed25519(key_bytes)) => {
            let pk = PublicKey::from_bytes_ed25519(&key_bytes).map_err(|e| {
                ErrorObject::owned(
                    INVALID_PARAMS_CODE,
                    format!("Failed to parse Ed25519 key: {}", e),
                    None::<()>,
                )
            })?;
            Ok(Key::Single(pk))
        }
        Some(EcdsaSecp256k1(key_bytes)) => {
            let pk = PublicKey::from_bytes_ecdsa(&key_bytes).map_err(|e| {
                ErrorObject::owned(
                    INVALID_PARAMS_CODE,
                    format!("Failed to parse ECDSA key: {}", e),
                    None::<()>,
                )
            })?;
            Ok(Key::Single(pk))
        }
        Some(KeyList(key_list_proto)) => {
            let mut key_list = hiero_sdk::KeyList::new();
            for proto_key_inner in key_list_proto.keys {
                let inner_key = parse_proto_key_to_hedera_key(proto_key_inner)?;
                key_list.keys.push(inner_key);
            }
            Ok(Key::KeyList(key_list))
        }
        Some(ThresholdKey(services::ThresholdKey {
            keys: Some(key_list_proto),
            threshold,
            ..
        })) => {
            let mut key_list = hiero_sdk::KeyList::new();
            for proto_key_inner in key_list_proto.keys {
                let inner_key = parse_proto_key_to_hedera_key(proto_key_inner)?;
                key_list.keys.push(inner_key);
            }
            key_list.threshold = Some(threshold);
            Ok(Key::KeyList(key_list))
        }
        _ => Err(ErrorObject::owned(
            INVALID_PARAMS_CODE,
            "Unsupported key type in protobuf",
            None::<()>,
        )),
    }
}
