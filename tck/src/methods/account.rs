use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountCreateTransaction,
    AccountId,
    AccountUpdateTransaction,
    EvmAddress,
    Hbar,
    PrivateKey,
};
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::ErrorObjectOwned;
use serde_json::Value;
use time::{
    Duration,
    OffsetDateTime,
};

use crate::errors::from_hedera_error;
use crate::helpers::{
    fill_common_transaction_params,
    get_hedera_key_with_protobuf,
};
use crate::methods::common::GLOBAL_SDK_CLIENT;
use crate::responses::{
    AccountCreateResponse,
    AccountUpdateResponse,
};

pub async fn create_account(
    key: Option<String>,
    initial_balance: Option<i64>,
    receiver_signature_required: Option<bool>,
    auto_renew_period: Option<i64>,
    memo: Option<String>,
    max_auto_token_associations: Option<i64>,
    staked_account_id: Option<String>,
    staked_node_id: Option<i64>,
    decline_staking_reward: Option<bool>,
    alias: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<AccountCreateResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut account_create_tx = AccountCreateTransaction::new();

    if let Some(key) = key {
        let key = get_hedera_key_with_protobuf(&key)?;
        account_create_tx.set_key_without_alias(key);
    }

    if let Some(initial_balance) = initial_balance {
        account_create_tx.initial_balance(Hbar::from_tinybars(initial_balance));
    }

    if let Some(receiver_signature_required) = receiver_signature_required {
        account_create_tx.receiver_signature_required(receiver_signature_required);
    }

    if let Some(auto_renew_period) = auto_renew_period {
        account_create_tx.auto_renew_period(Duration::seconds(auto_renew_period));
    }

    if let Some(memo) = memo {
        account_create_tx.account_memo(memo);
    }

    if let Some(max_auto_token_associations) = max_auto_token_associations {
        account_create_tx.max_automatic_token_associations(max_auto_token_associations as i32);
    }

    if let Some(staked_account_id) = staked_account_id {
        account_create_tx.staked_account_id(AccountId::from_str(&staked_account_id).map_err(
            |e| {
                jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            },
        )?);
    }

    if let Some(alias) = alias {
        account_create_tx.alias(EvmAddress::from_str(&alias).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
        })?);
    }

    if let Some(staked_node_id) = staked_node_id {
        account_create_tx.staked_node_id(staked_node_id as u64);
    }

    if let Some(decline_staking_reward) = decline_staking_reward {
        account_create_tx.decline_staking_reward(decline_staking_reward);
    }

    if let Some(common_transaction_params) = common_transaction_params {
        let _ = fill_common_transaction_params(&mut account_create_tx, &common_transaction_params, &client);
        account_create_tx.freeze_with(&client).unwrap();

        if let Some(signers) = common_transaction_params.get("signers") {
            if let Value::Array(signers) = signers {
                for signer in signers {
                    if let Value::String(signer_str) = signer {
                        account_create_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                    }
                }
            }
        }
    }

    let tx_response = account_create_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(AccountCreateResponse {
        account_id: tx_receipt.account_id.unwrap().to_string(),
        status: tx_receipt.status.as_str_name().to_string(),
    })
}

pub async fn update_account(
    account_id: Option<String>,
    key: Option<String>,
    auto_renew_period: Option<i64>,
    expiration_time: Option<i64>,
    receiver_signature_required: Option<bool>,
    memo: Option<String>,
    max_auto_token_associations: Option<i64>,
    staked_account_id: Option<String>,
    staked_node_id: Option<i64>,
    decline_staking_reward: Option<bool>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<AccountUpdateResponse, ErrorObjectOwned> {
    let client = {
        let guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    INTERNAL_ERROR_CODE,
                    "Client not initialized".to_string(),
                    None::<()>,
                )
            })?
            .clone()
    };

    let mut account_update_tx = AccountUpdateTransaction::new();

    if let Some(account_id) = account_id {
        account_update_tx.account_id(account_id.parse().unwrap());
    }

    if let Some(key) = key {
        let key = get_hedera_key_with_protobuf(&key)?;
        account_update_tx.key(key);
    }

    if let Some(receiver_signature_required) = receiver_signature_required {
        account_update_tx.receiver_signature_required(receiver_signature_required);
    }

    if let Some(auto_renew_period) = auto_renew_period {
        account_update_tx.auto_renew_period(Duration::seconds(auto_renew_period));
    }

    if let Some(expiration_time) = expiration_time {
        account_update_tx.expiration_time(
            OffsetDateTime::from_unix_timestamp(expiration_time).map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            })?,
        );
    }

    if let Some(memo) = memo {
        account_update_tx.account_memo(memo);
    }

    if let Some(max_auto_token_associations) = max_auto_token_associations {
        account_update_tx.max_automatic_token_associations(max_auto_token_associations as i32);
    }

    if let Some(staked_account_id) = staked_account_id {
        account_update_tx.staked_account_id(AccountId::from_str(&staked_account_id).map_err(
            |e| {
                jsonrpsee::types::ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
            },
        )?);
    }

    if let Some(staked_node_id) = staked_node_id {
        account_update_tx.staked_node_id(staked_node_id as u64);
    }

    if let Some(decline_staking_reward) = decline_staking_reward {
        account_update_tx.decline_staking_reward(decline_staking_reward);
    }

    if let Some(common_transaction_params) = common_transaction_params {
        let _ = fill_common_transaction_params(&mut account_update_tx, &common_transaction_params, &client);
        account_update_tx.freeze_with(&client).unwrap();

        if let Some(signers) = common_transaction_params.get("signers") {
            if let Value::Array(signers) = signers {
                for signer in signers {
                    if let Value::String(signer_str) = signer {
                        account_update_tx.sign(PrivateKey::from_str_der(signer_str).unwrap());
                    }
                }
            }
        }
    }

    let tx_response = account_update_tx.execute(&client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(&client).await.map_err(|e| from_hedera_error(e))?;

    Ok(AccountUpdateResponse { status: tx_receipt.status.as_str_name().to_string() })
}
