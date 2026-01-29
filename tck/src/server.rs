// RPC implementation lives in methods::RpcServerImpl (see methods/mod.rs).
// This module only keeps build_transfer_tx_from_value for use by schedule if re-added.

use std::str::FromStr;

use hiero_sdk::{
    AccountId,
    EvmAddress,
    Hbar,
    NftId,
    TokenId,
    TransferTransaction,
};
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use serde_json::Value;

pub(crate) fn build_transfer_tx_from_value(
    params: &Value,
) -> Result<TransferTransaction, ErrorObjectOwned> {
    let mut tx = TransferTransaction::new();

    let transfers = params.get("transfers").and_then(Value::as_array).ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "transfers array is required for transferCrypto".to_string(),
            None::<()>,
        )
    })?;

    if transfers.is_empty() {
        return Err(ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "No transfers provided".to_string(),
            None::<()>,
        ));
    }

    for t in transfers {
        if let Value::Object(obj) = t {
            let is_approved = obj.get("approved").and_then(Value::as_bool).unwrap_or(false);

            // Handle HBAR transfers
            if let Some(hbar_obj) = obj.get("hbar").and_then(Value::as_object) {
                let amount_i64 = hbar_obj
                    .get("amount")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "amount is required in hbar transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let amount = Hbar::from_tinybars(amount_i64);

                if let Some(account_id_str) = hbar_obj.get("accountId").and_then(Value::as_str) {
                    let account_id = AccountId::from_str(account_id_str).map_err(|e| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid account ID: {e}"),
                            None::<()>,
                        )
                    })?;

                    if is_approved {
                        tx.approved_hbar_transfer(account_id, amount);
                    } else {
                        tx.hbar_transfer(account_id, amount);
                    }
                } else if let Some(evm_address_str) =
                    hbar_obj.get("evmAddress").and_then(Value::as_str)
                {
                    let evm_address = EvmAddress::from_str(evm_address_str).map_err(|e| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid EVM address: {e}"),
                            None::<()>,
                        )
                    })?;
                    let account_id = AccountId::from_evm_address(&evm_address, 0, 0);

                    if is_approved {
                        tx.approved_hbar_transfer(account_id, amount);
                    } else {
                        tx.hbar_transfer(account_id, amount);
                    }
                } else {
                    return Err(ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        "Either accountId or evmAddress is required in hbar transfers".to_string(),
                        None::<()>,
                    ));
                }
            }
            // Handle token transfers
            else if let Some(token_obj) = obj.get("token").and_then(Value::as_object) {
                let account_id_str =
                    token_obj.get("accountId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "accountId is required in token transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let account_id = AccountId::from_str(account_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid account ID: {e}"),
                        None::<()>,
                    )
                })?;

                let token_id_str =
                    token_obj.get("tokenId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in token transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid token ID: {e}"),
                        None::<()>,
                    )
                })?;

                let amount = token_obj
                    .get("amount")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "amount is required in token transfers".to_string(),
                            None::<()>,
                        )
                    })?;

                let decimals = token_obj.get("decimals").and_then(Value::as_u64).map(|d| d as u32);

                if is_approved {
                    tx.approved_token_transfer(token_id, account_id, amount);
                } else if let Some(dec) = decimals {
                    tx.token_transfer_with_decimals(token_id, account_id, amount, dec);
                } else {
                    tx.token_transfer(token_id, account_id, amount);
                }
            }
            // Handle NFT transfers
            else if let Some(nft_obj) = obj.get("nft").and_then(Value::as_object) {
                let sender_account_id_str =
                    nft_obj.get("senderAccountId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "senderAccountId is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let sender_account_id =
                    AccountId::from_str(sender_account_id_str).map_err(|e| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid sender account ID: {e}"),
                            None::<()>,
                        )
                    })?;

                let receiver_account_id_str =
                    nft_obj.get("receiverAccountId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "receiverAccountId is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let receiver_account_id =
                    AccountId::from_str(receiver_account_id_str).map_err(|e| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            format!("Invalid receiver account ID: {e}"),
                            None::<()>,
                        )
                    })?;

                let token_id_str =
                    nft_obj.get("tokenId").and_then(Value::as_str).ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "tokenId is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;
                let token_id = TokenId::from_str(token_id_str).map_err(|e| {
                    ErrorObject::owned(
                        INTERNAL_ERROR_CODE,
                        format!("Invalid token ID: {e}"),
                        None::<()>,
                    )
                })?;

                let serial_number = nft_obj
                    .get("serialNumber")
                    .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            INTERNAL_ERROR_CODE,
                            "serialNumber is required in NFT transfers".to_string(),
                            None::<()>,
                        )
                    })?;

                let nft_id = NftId::from((token_id, serial_number as u64));

                if is_approved {
                    tx.approved_nft_transfer(nft_id, sender_account_id, receiver_account_id);
                } else {
                    tx.nft_transfer(nft_id, sender_account_id, receiver_account_id);
                }
            }
        }
    }

    Ok(tx)
}
