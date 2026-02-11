use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountId,
    Client,
    PrivateKey,
};
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::ErrorObjectOwned;
use serde_json::Value;

use crate::helpers::generate_key_helper;
use crate::responses::GenerateKeyResponse;

use crate::methods::common::GLOBAL_SDK_CLIENT;

pub fn setup(
    operator_account_id: Option<String>,
    operator_private_key: Option<String>,
    node_ip: Option<String>,
    node_account_id: Option<String>,
    mirror_network_ip: Option<String>,
) -> Result<String, ErrorObjectOwned> {
    let mut network: HashMap<String, AccountId> = HashMap::new();

    // Client setup, if the network is not set, it will be created using testnet.
    // If the network is manually set, the network will be configured using the
    // provided ips and account id.
    let client = match (node_ip, node_account_id, mirror_network_ip) {
        (Some(node_ip), Some(node_account_id), Some(mirror_network_ip)) => {
            let account_id = AccountId::from_str(node_account_id.as_str()).map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?;
            network.insert(node_ip, account_id);

            let client = Client::for_network(network).map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?;
            client.set_mirror_network([mirror_network_ip]);
            client
        }
        (None, None, None) => Client::for_testnet(),
        _ => {
            return Err(jsonrpsee::types::ErrorObject::borrowed(
                INTERNAL_ERROR_CODE,
                "Failed to setup client",
                None,
            ))
        }
    };

    let operator_id = if let Some(operator_account_id) = operator_account_id {
        AccountId::from_str(operator_account_id.as_str())
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?
    } else {
        return Err(jsonrpsee::types::ErrorObject::borrowed(
            INTERNAL_ERROR_CODE,
            "Missing operator account id",
            None,
        ));
    };

    let operator_key = if let Some(operator_private_key) = operator_private_key {
        PrivateKey::from_str(operator_private_key.as_str())
            .map_err(|e| jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?
    } else {
        return Err(jsonrpsee::types::ErrorObject::borrowed(
            INTERNAL_ERROR_CODE,
            "Missing operator private key",
            None,
        ));
    };

    client.set_operator(operator_id, operator_key);

    let mut global_client = GLOBAL_SDK_CLIENT.lock().unwrap();
    *global_client = Some(client);

    Ok("SUCCESS".to_owned())
}

pub fn reset() -> Result<HashMap<String, String>, ErrorObjectOwned> {
    let mut global_client = GLOBAL_SDK_CLIENT.lock().unwrap();
    *global_client = None;
    Ok(HashMap::from([("status".to_string(), "SUCCESS".to_string())]))
}

pub fn generate_key(
    _type: String,
    from_key: Option<String>,
    threshold: Option<i32>,
    keys: Option<Value>,
) -> Result<GenerateKeyResponse, ErrorObjectOwned> {
    let mut private_keys: Vec<Value> = Vec::new();

    let key = generate_key_helper(_type, from_key, threshold, keys, &mut private_keys, false)?;

    Ok(GenerateKeyResponse { key, private_keys })
}
