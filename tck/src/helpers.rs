use std::collections::HashMap;
use std::str::FromStr;

use hex::{
    decode as hex_decode,
    ToHex,
};
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

pub(crate) fn fill_common_transaction_params<D: hiero_sdk::ValidateChecksums>(
    transaction: &mut Transaction<D>,
    common_transaction_params: &HashMap<String, Value>,
    client: &Client,
) {
    if let Some(Value::String(transaction_id)) = common_transaction_params.get("transactionId") {
        transaction.transaction_id(TransactionId::from_str(transaction_id).unwrap());
    }

    if let Some(Value::String(node_id)) = common_transaction_params.get("nodeId") {
        transaction.node_account_ids([AccountId::from_str(node_id).unwrap()]);
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

    if let Some(Value::String(duration)) = common_transaction_params.get("transactionValidDuration")
    {
        transaction.transaction_valid_duration(Duration::seconds(duration.parse::<i64>().unwrap()));
    }

    if let Some(Value::String(memo)) = common_transaction_params.get("memo") {
        transaction.transaction_memo(memo);
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
    let key_type = KeyType::from_str(&_type)?;

    // Validate parameters
    validate_key_params(&key_type, from_key.is_some(), threshold, keys.is_some())?;

    match key_type {
        KeyType::Ed25519PrivateKeyType | KeyType::EcdsaSecp256k1PrivateKeyType => {
            generate_private_key(key_type, private_keys, is_list)
        }
        KeyType::Ed25519PublicKeyType | KeyType::EcdsaSecp256k1PublicKeyType => {
            generate_public_key(key_type, from_key, private_keys, is_list)
        }
        KeyType::KeyListType | KeyType::ThresholdKeyType => {
            generate_key_list(key_type, keys.unwrap(), threshold, private_keys)
        }
        KeyType::EvmAddressType => generate_evm_address(from_key),
    }
}

fn validate_key_params(
    key_type: &KeyType,
    has_from_key: bool,
    threshold: Option<i32>,
    has_keys: bool,
) -> Result<(), ErrorObjectOwned> {
    if has_from_key
        && *key_type != KeyType::Ed25519PublicKeyType
        && *key_type != KeyType::EcdsaSecp256k1PublicKeyType
        && *key_type != KeyType::EvmAddressType
    {
        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: fromKey MUST NOT be provided for types other than ed25519PublicKey, ecdsaSecp256k1PublicKey, or evmAddress.",
            None,
        ));
    }

    if threshold.is_some() && *key_type != KeyType::ThresholdKeyType {
        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: threshold MUST NOT be provided for types other than thresholdKey.",
            None,
        ));
    }

    if threshold.is_none() && *key_type == KeyType::ThresholdKeyType {
        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: threshold MUST be provided for thresholdKey types.",
            None,
        ));
    }

    if has_keys && *key_type != KeyType::ThresholdKeyType && *key_type != KeyType::KeyListType {
        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: keys MUST NOT be provided for types other than keyList or thresholdKey.",
            None,
        ));
    }

    if !has_keys && (*key_type == KeyType::ThresholdKeyType || *key_type == KeyType::KeyListType) {
        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: keys MUST be provided for keyList and thresholdKey types.",
            None,
        ));
    }

    Ok(())
}

fn generate_private_key(
    key_type: KeyType,
    private_keys: &mut Vec<Value>,
    is_list: bool,
) -> Result<String, ErrorObjectOwned> {
    let key = match key_type {
        KeyType::Ed25519PrivateKeyType => PrivateKey::generate_ed25519().to_string_der(),
        KeyType::EcdsaSecp256k1PrivateKeyType => PrivateKey::generate_ecdsa().to_string_der(),
        _ => unreachable!(),
    };

    if is_list {
        private_keys.push(Value::String(key.clone()));
    }

    Ok(key)
}

fn generate_public_key(
    key_type: KeyType,
    from_key: Option<String>,
    private_keys: &mut Vec<Value>,
    is_list: bool,
) -> Result<String, ErrorObjectOwned> {
    if let Some(from_key) = from_key {
        return parse_public_key_from_string(key_type, &from_key);
    }

    let private_key = match key_type {
        KeyType::Ed25519PublicKeyType => PrivateKey::generate_ed25519(),
        KeyType::EcdsaSecp256k1PublicKeyType => PrivateKey::generate_ecdsa(),
        _ => unreachable!(),
    };

    if is_list {
        private_keys.push(Value::String(private_key.to_string_der()));
    }

    Ok(private_key.public_key().to_string_der())
}

fn parse_public_key_from_string(
    key_type: KeyType,
    from_key: &str,
) -> Result<String, ErrorObjectOwned> {
    // Try parsing as private key first
    if let Ok(private_key) = PrivateKey::from_str_der(from_key) {
        return Ok(private_key.public_key().to_string_der());
    }

    // Try parsing as public key
    if let Ok(public_key) = PublicKey::from_str_der(from_key) {
        return Ok(public_key.to_string_der());
    }

    // Try raw formats as fallback
    match key_type {
        KeyType::Ed25519PublicKeyType => {
            if let Ok(public_key) = PublicKey::from_str_ed25519(from_key) {
                return Ok(public_key.to_string_der());
            }
        }
        KeyType::EcdsaSecp256k1PublicKeyType => {
            if let Ok(public_key) = PublicKey::from_str_ecdsa(from_key) {
                return Ok(public_key.to_string_der());
            }
        }
        _ => {}
    }

    Err(ErrorObject::owned(
        INVALID_PARAMS_CODE,
        format!("generateKey: could not produce {:?} from fromKey", key_type),
        None::<()>,
    ))
}

fn generate_key_list(
    key_type: KeyType,
    keys: Value,
    threshold: Option<i32>,
    private_keys: &mut Vec<Value>,
) -> Result<String, ErrorObjectOwned> {
    let mut key_list = KeyList::new();

    if let Value::Array(key_array) = keys {
        for key_obj in key_array {
            let from_key = key_obj.get("fromKey").and_then(|v| v.as_str()).map(|s| s.to_string());

            let key_type_str = key_obj["type"].as_str().unwrap().to_string();

            let generated_key = generate_key_helper(
                key_type_str,
                from_key,
                None,
                key_obj.get("keys").map(|v| v.clone()),
                private_keys,
                true,
            )?;

            // Always use protobuf-aware parsing since generated_key might be a protobuf-encoded KeyList
            // (e.g., when fromKey is a KeyList)
            let parsed_key = get_hedera_key_with_protobuf(&generated_key)?;

            key_list.keys.push(parsed_key);
        }
    }

    if key_type == KeyType::ThresholdKeyType {
        key_list.threshold = Some(threshold.unwrap() as u32);
    }

    Ok(Key::KeyList(key_list).to_bytes().encode_hex())
}

fn generate_evm_address(from_key: Option<String>) -> Result<String, ErrorObjectOwned> {
    if let Some(from_key) = from_key {
        // Try as private key first
        if let Ok(private_key) = PrivateKey::from_str_ecdsa(&from_key) {
            return Ok(private_key.public_key().to_evm_address().unwrap().to_string());
        }

        // Try as public key
        if let Ok(public_key) = PublicKey::from_str_ecdsa(&from_key) {
            return Ok(public_key.to_evm_address().unwrap().to_string());
        }

        return Err(ErrorObject::borrowed(
            INVALID_PARAMS_CODE,
            "generateKey: fromKey for evmAddress MUST be an ECDSAsecp256k1 private or public key.",
            None,
        ));
    }

    Ok(PrivateKey::generate_ecdsa().public_key().to_evm_address().unwrap().to_string())
}

// ============================================================================
// Key Parsing Functions
// ============================================================================

pub(crate) fn get_hedera_key_with_protobuf(key: &str) -> Result<Key, ErrorObjectOwned> {
    // First try normal key parsing
    if let Ok(k) = get_hedera_key(key) {
        return Ok(k);
    }

    // Try parsing as protobuf
    let bytes = match hex_decode(key.strip_prefix("0x").unwrap_or(key)) {
        Ok(b) => b,
        Err(_) => {
            // Not valid hex, return original error
            return get_hedera_key(key);
        }
    };

    let pb_key = match services::Key::decode(bytes.as_slice()) {
        Ok(k) => k,
        Err(e) => {
            // Failed to decode as protobuf - return a more informative error
            return Err(ErrorObject::owned(
                -32603,
                format!(
                    "generateKey: failed to decode protobuf Key (length: {}, hex bytes: {}, decode error: {}). Trying fallback...",
                    key.len(), bytes.len(), e
                ),
                None::<()>,
            ));
        }
    };

    // If protobuf parsing succeeds, use it; otherwise return a more specific error
    parse_protobuf_key(pb_key, key.len(), bytes.len())
}

fn parse_protobuf_key(
    pb_key: services::Key,
    key_len: usize,
    bytes_len: usize,
) -> Result<Key, ErrorObjectOwned> {
    use services::key::Key::*;

    match pb_key.key {
        Some(Ed25519(b)) => PublicKey::from_bytes_ed25519(&b)
            .map(Key::Single)
            .map_err(|_| protobuf_parse_error("Ed25519", key_len, bytes_len)),
        Some(EcdsaSecp256k1(b)) => PublicKey::from_bytes_ecdsa(&b)
            .map(Key::Single)
            .map_err(|_| protobuf_parse_error("ECDSA", key_len, bytes_len)),
        Some(KeyList(kl)) => parse_protobuf_key_list(kl),
        Some(ThresholdKey(tk)) => parse_protobuf_threshold_key(tk),
        None => Err(ErrorObject::owned(
            -32603,
            format!(
                "generateKey: decoded protobuf Key but key field is None (length: {}, hex bytes: {})",
                key_len, bytes_len
            ),
            None::<()>,
        )),
        _ => Err(ErrorObject::owned(
            -32603,
            format!(
                "generateKey: decoded protobuf Key but key type is not supported (length: {}, hex bytes: {})",
                key_len, bytes_len
            ),
            None::<()>,
        )),
    }
}

fn protobuf_parse_error(key_type: &str, key_len: usize, bytes_len: usize) -> ErrorObjectOwned {
    ErrorObject::owned(
        -32603,
        format!(
            "generateKey: failed to parse {} public key from protobuf (length: {}, hex bytes: {})",
            key_type, key_len, bytes_len
        ),
        None::<()>,
    )
}

fn parse_protobuf_key_list(kl: services::KeyList) -> Result<Key, ErrorObjectOwned> {
    let mut key_list = hiero_sdk::KeyList::new();

    for pb_key in kl.keys {
        match parse_single_protobuf_key(&pb_key) {
            Ok(key) => key_list.keys.push(key),
            Err(e) => {
                // Log but continue - don't fail entire KeyList if one key fails
                // In production you might want to return the error instead
                return Err(e);
            }
        }
    }

    Ok(Key::KeyList(key_list))
}

fn parse_protobuf_threshold_key(tk: services::ThresholdKey) -> Result<Key, ErrorObjectOwned> {
    let mut key_list = hiero_sdk::KeyList::new();
    key_list.threshold = Some(tk.threshold);

    if let Some(kl) = tk.keys {
        for pb_key in kl.keys {
            match parse_single_protobuf_key(&pb_key) {
                Ok(key) => key_list.keys.push(key),
                Err(e) => return Err(e),
            }
        }
    }

    Ok(Key::KeyList(key_list))
}

fn parse_single_protobuf_key(pb_key: &services::Key) -> Result<Key, ErrorObjectOwned> {
    use services::key::Key::*;

    match &pb_key.key {
        Some(Ed25519(b)) => PublicKey::from_bytes_ed25519(b)
            .map(Key::Single)
            .map_err(|_| ErrorObject::borrowed(-32603, "Failed to parse Ed25519 key", None)),
        Some(EcdsaSecp256k1(b)) => PublicKey::from_bytes_ecdsa(b)
            .map(Key::Single)
            .map_err(|_| ErrorObject::borrowed(-32603, "Failed to parse ECDSA key", None)),
        Some(KeyList(inner_kl)) => parse_protobuf_key_list(inner_kl.clone()),
        Some(ThresholdKey(inner_tk)) => parse_protobuf_threshold_key(inner_tk.clone()),
        _ => Err(ErrorObject::borrowed(-32603, "Unsupported key type in KeyList", None)),
    }
}

pub(crate) fn get_hedera_key(key: &str) -> Result<Key, ErrorObjectOwned> {
    if key.is_empty() {
        return Err(ErrorObject::borrowed(-32603, "generateKey: key string is empty.", None));
    }

    // Try DER-encoded private key
    if let Ok(pk) = PrivateKey::from_str_der(key) {
        return Ok(Key::Single(pk.public_key()));
    }

    // Try DER-encoded public key
    if let Ok(pk) = PublicKey::from_str_der(key) {
        return Ok(Key::Single(pk));
    }

    // Try raw Ed25519 public key
    if let Ok(pk) = PublicKey::from_str_ed25519(key) {
        return Ok(Key::Single(pk));
    }

    // Try raw ECDSA public key
    if let Ok(pk) = PublicKey::from_str_ecdsa(key) {
        return Ok(Key::Single(pk));
    }

    // All parsing attempts failed
    let key_len = key.len();
    let hex_len =
        hex_decode(key.strip_prefix("0x").unwrap_or(key)).map(|bytes| bytes.len()).unwrap_or(0);

    if hex_len == 0 {
        return Err(ErrorObject::borrowed(
            -32603,
            "generateKey: key is not a valid hex string.",
            None,
        ));
    }

    Err(ErrorObject::owned(
        -32603,
        format!(
            "generateKey: could not parse key (length: {}, hex bytes: {}). Expected DER-encoded private key, DER-encoded public key, or raw hex public key.",
            key_len, hex_len
        ),
        None::<()>,
    ))
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
