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
use jsonrpsee::types::error::INVALID_PARAMS_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
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
                let public_key = PublicKey::from_str_ed25519(key).map_err(|_| {
                    ErrorObject::borrowed(-32603, "generateKey: fromKey is invalid.", None)
                })?;

                Ok(public_key.into())
            }
        },
    }
}
