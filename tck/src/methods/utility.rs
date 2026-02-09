use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{
    Arc,
    Mutex,
};

use hiero_sdk::{
    AccountId,
    Client,
    PrivateKey,
};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use once_cell::sync::Lazy;
use serde_json::Value;

use crate::common::internal_error;
use crate::helpers::generate_key_helper;
use crate::responses::GenerateKeyResponse;

#[rpc(server, client)]
pub trait UtilityRpc {
    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/setup.md#setup
    */
    #[method(name = "setup")]
    fn setup(
        &self,
        operator_account_id: Option<String>,
        operator_private_key: Option<String>,
        node_ip: Option<String>,
        node_account_id: Option<String>,
        mirror_network_ip: Option<String>,
    ) -> Result<String, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/reset.md#reset
    */
    #[method(name = "reset")]
    fn reset(&self) -> Result<HashMap<String, String>, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/generate-key.md#generateKey
    */
    #[method(name = "generateKey")]
    fn generate_key(
        &self,
        _type: String,
        from_key: Option<String>,
        threshold: Option<i32>,
        keys: Option<Value>,
    ) -> Result<GenerateKeyResponse, ErrorObjectOwned>;

    /// Updates the operator on an existing client.
    /// Use this to switch payers within the same session without recreating the client.
    #[method(name = "setOperator")]
    fn set_operator(
        &self,
        operator_account_id: String,
        operator_private_key: String,
    ) -> Result<HashMap<String, String>, ErrorObjectOwned>;
}

static GLOBAL_SDK_CLIENT: Lazy<Arc<Mutex<Option<Client>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn get_client() -> Result<Client, ErrorObjectOwned> {
    let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
    guard
        .as_ref()
        .ok_or_else(|| {
            ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Client not initialized".to_string(),
                None::<()>,
            )
        })
        .cloned()
}

pub fn setup(
    operator_account_id: Option<String>,
    operator_private_key: Option<String>,
    node_ip: Option<String>,
    node_account_id: Option<String>,
    mirror_network_ip: Option<String>,
) -> Result<String, ErrorObjectOwned> {
    let mut network: HashMap<String, AccountId> = HashMap::new();
    tracing::debug!("setup request params: operatorAccountId: {:?}, operatorPrivateKey: {:?}, nodeIp: {:?}, nodeAccountId: {:?}, mirrorNetworkIp: {:?}", operator_account_id, operator_private_key, node_ip, node_account_id, mirror_network_ip);

    // Client setup, if the network is not set, it will be created using testnet.
    // If the network is manually set, the network will be configured using the
    // provided ips and account id.
    let client = match (node_ip, node_account_id, mirror_network_ip) {
        (Some(node_ip), Some(node_account_id), Some(mirror_network_ip)) => {
            let account_id =
                AccountId::from_str(node_account_id.as_str()).map_err(internal_error)?;
            network.insert(node_ip, account_id);

            let client = Client::for_network(network).map_err(internal_error)?;
            client.set_mirror_network([mirror_network_ip]);
            client
        }
        (None, None, None) => Client::for_testnet(),
        _ => {
            return Err(ErrorObject::borrowed(INTERNAL_ERROR_CODE, "Failed to setup client", None))
        }
    };

    let operator_id = if let Some(operator_account_id) = operator_account_id {
        AccountId::from_str(operator_account_id.as_str()).map_err(internal_error)?
    } else {
        return Err(ErrorObject::borrowed(
            INTERNAL_ERROR_CODE,
            "Missing operator account id",
            None,
        ));
    };

    let operator_key = if let Some(operator_private_key) = operator_private_key {
        PrivateKey::from_str(operator_private_key.as_str()).map_err(internal_error)?
    } else {
        return Err(ErrorObject::borrowed(
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
    Ok(HashMap::from([("status".to_string(), "SUCCESS".to_string())].to_owned()))
}

pub fn set_operator(
    operator_account_id: String,
    operator_private_key: String,
) -> Result<HashMap<String, String>, ErrorObjectOwned> {
    // Get the existing client
    let client = get_client()?;

    // Parse the operator account ID
    let operator_id = AccountId::from_str(&operator_account_id).map_err(internal_error)?;

    // Parse the operator private key
    let operator_key = PrivateKey::from_str(&operator_private_key).map_err(internal_error)?;

    // Update the operator on the client
    client.set_operator(operator_id, operator_key);

    // Update the global client
    let mut global_client = GLOBAL_SDK_CLIENT.lock().unwrap();
    *global_client = Some(client);

    Ok(HashMap::from([
        ("status".to_string(), "SUCCESS".to_string()),
        ("message".to_string(), "Operator updated for default client".to_string()),
    ]))
}

pub fn generate_key(
    _type: String,
    from_key: Option<String>,
    threshold: Option<i32>,
    keys: Option<Value>,
) -> Result<GenerateKeyResponse, ErrorObjectOwned> {
    let mut private_keys: Vec<Value> = Vec::new();

    let key = generate_key_helper(_type, from_key, threshold, keys, &mut private_keys, false)?;

    Ok(GenerateKeyResponse { key: key, private_keys: private_keys })
}
