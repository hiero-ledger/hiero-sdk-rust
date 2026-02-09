use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    AccountId,
    Client,
    Hbar,
    ScheduleCreateTransaction,
    ScheduleDeleteTransaction,
    ScheduleId,
    ScheduleInfoQuery,
    ScheduleSignTransaction,
};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use serde_json::Value;
use time::OffsetDateTime;

use crate::common::{
    internal_error,
    mock_consensus_error,
};
use crate::errors::from_hedera_error;
use crate::helpers::{
    fill_common_transaction_params,
    get_hedera_key,
    key_to_der_string,
};
use crate::responses::{
    ScheduleInfoResponse,
    ScheduleResponse,
};

#[rpc(server, client)]
pub trait ScheduleRpc {
    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/schedule-service/scheduleCreateTransaction.md#createSchedule
    */
    #[method(name = "createSchedule")]
    async fn create_schedule(
        &self,
        scheduled_transaction: Option<Value>,
        memo: Option<String>,
        admin_key: Option<String>,
        payer_account_id: Option<String>,
        expiration_time: Option<String>,
        wait_for_expiry: Option<bool>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/schedule-service/scheduleSignTransaction.md#signSchedule
    */
    #[method(name = "signSchedule")]
    async fn sign_schedule(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned>;

    /*
    / Specification:
    / https://github.com/hiero-ledger/hiero-sdk-tck/blob/main/test-specifications/schedule-service/scheduleDeleteTransaction.md#deleteSchedule
    */
    #[method(name = "deleteSchedule")]
    async fn delete_schedule(
        &self,
        schedule_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<ScheduleResponse, ErrorObjectOwned>;

    #[method(name = "getScheduleInfo")]
    async fn get_schedule_info(
        &self,
        schedule_id: Option<String>,
        query_payment: Option<String>,
        max_query_payment: Option<String>,
        get_cost: Option<bool>,
    ) -> Result<ScheduleInfoResponse, ErrorObjectOwned>;
}

pub async fn create_schedule(
    client: &Client,
    scheduled_transaction: Option<Value>,
    memo: Option<String>,
    admin_key: Option<String>,
    payer_account_id: Option<String>,
    expiration_time: Option<String>,
    wait_for_expiry: Option<bool>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<ScheduleResponse, ErrorObjectOwned> {
    let mut schedule_create_tx = ScheduleCreateTransaction::new();

    if let Some(scheduled_transaction) = &scheduled_transaction {
        build_scheduled_transaction(scheduled_transaction, client, &mut schedule_create_tx)?;
    }

    if let Some(memo) = memo {
        schedule_create_tx.schedule_memo(memo);
    }

    if let Some(admin_key) = admin_key {
        let key = get_hedera_key(&admin_key)?;
        schedule_create_tx.admin_key(key);
    }

    if let Some(payer_account_id) = payer_account_id {
        schedule_create_tx
            .payer_account_id(AccountId::from_str(&payer_account_id).map_err(internal_error)?);
    }

    if let Some(expiration_time) = expiration_time {
        let ts = expiration_time.parse::<i64>().map_err(internal_error)?;

        if ts < 0 {
            return Err(mock_consensus_error(
                "SCHEDULE_EXPIRATION_TIME_MUST_BE_HIGHER_THAN_CONSENSUS_TIME",
                "Expiration time must be positive",
            ));
        }

        schedule_create_tx.expiration_time(OffsetDateTime::from_unix_timestamp(ts).map_err(
            |e| mock_consensus_error("SCHEDULE_EXPIRATION_TIME_TOO_FAR_IN_FUTURE", &e.to_string()),
        )?);
    }

    if let Some(wait_for_expiry) = wait_for_expiry {
        schedule_create_tx.wait_for_expiry(wait_for_expiry);
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(&mut schedule_create_tx, &common_transaction_params, client);
    }

    let tx_response = schedule_create_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(ScheduleResponse {
        schedule_id: tx_receipt.schedule_id.map(|id| id.to_string()),
        transaction_id: tx_receipt.scheduled_transaction_id.map(|id| id.to_string()),
        status: tx_receipt.status.as_str_name().to_string(),
    })
}

pub async fn sign_schedule(
    client: &Client,
    schedule_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<ScheduleResponse, ErrorObjectOwned> {
    let mut schedule_sign_tx = ScheduleSignTransaction::new();

    if let Some(schedule_id) = schedule_id {
        let sid = ScheduleId::from_str(&schedule_id).map_err(internal_error)?;
        schedule_sign_tx.schedule_id(sid);
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(&mut schedule_sign_tx, &common_transaction_params, client);
    }

    let tx_response = schedule_sign_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(ScheduleResponse {
        schedule_id: None,
        transaction_id: None,
        status: tx_receipt.status.as_str_name().to_string(),
    })
}

pub async fn delete_schedule(
    client: &Client,
    schedule_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<ScheduleResponse, ErrorObjectOwned> {
    let mut schedule_delete_tx = ScheduleDeleteTransaction::new();

    if let Some(schedule_id) = schedule_id {
        let sid = ScheduleId::from_str(&schedule_id).map_err(internal_error)?;
        schedule_delete_tx.schedule_id(sid);
    }

    if let Some(common_transaction_params) = common_transaction_params {
        fill_common_transaction_params(&mut schedule_delete_tx, &common_transaction_params, client);
    }

    let tx_response = schedule_delete_tx.execute(client).await.map_err(|e| from_hedera_error(e))?;

    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(ScheduleResponse {
        schedule_id: None,
        transaction_id: None,
        status: tx_receipt.status.as_str_name().to_string(),
    })
}

pub async fn get_schedule_info(
    client: &Client,
    schedule_id: Option<String>,
    query_payment: Option<String>,
    max_query_payment: Option<String>,
    get_cost: Option<bool>,
) -> Result<ScheduleInfoResponse, ErrorObjectOwned> {
    let mut query = ScheduleInfoQuery::new();

    if let Some(schedule_id) = schedule_id {
        query.schedule_id(ScheduleId::from_str(&schedule_id).map_err(internal_error)?);
    }

    if let Some(query_payment) = query_payment {
        let amount = query_payment.parse::<i64>().map_err(internal_error)?;
        query.payment_amount(Hbar::from_tinybars(amount));
    }

    if let Some(max_query_payment) = max_query_payment {
        let amount = max_query_payment.parse::<i64>().map_err(internal_error)?;
        query.max_payment_amount(Hbar::from_tinybars(amount));
    }

    // If getCost is requested, return just the cost
    if get_cost.unwrap_or(false) {
        let cost = query.get_cost(client).await.map_err(|e| from_hedera_error(e))?;
        return Ok(ScheduleInfoResponse {
            schedule_id: None,
            creator_account_id: None,
            payer_account_id: None,
            admin_key: None,
            signers: None,
            schedule_memo: None,
            expiration_time: None,
            executed: None,
            deleted: None,
            scheduled_transaction_id: None,
            wait_for_expiry: None,
            cost: Some(cost.to_tinybars().to_string()),
        });
    }

    let info = query.execute(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(ScheduleInfoResponse {
        schedule_id: Some(info.schedule_id.to_string()),
        creator_account_id: Some(info.creator_account_id.to_string()),
        payer_account_id: info.payer_account_id.map(|id| id.to_string()),
        admin_key: info.admin_key.map(|k| key_to_der_string(&k)),
        signers: Some(info.signatories.keys.iter().map(|k| key_to_der_string(k)).collect()),
        schedule_memo: Some(info.memo.clone()),
        expiration_time: info.expiration_time.map(|t| t.unix_timestamp().to_string()),
        executed: info.executed_at.map(|t| t.unix_timestamp().to_string()),
        deleted: info.deleted_at.map(|t| t.unix_timestamp().to_string()),
        scheduled_transaction_id: Some(info.scheduled_transaction_id.to_string()),
        wait_for_expiry: Some(info.wait_for_expiry),
        cost: None,
    })
}

pub fn build_scheduled_transaction(
    scheduled_tx: &Value,
    _client: &Client,
    schedule_tx: &mut ScheduleCreateTransaction,
) -> Result<(), ErrorObjectOwned> {
    let method = scheduled_tx.get("method").and_then(Value::as_str).ok_or_else(|| {
        ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            "scheduledTransaction.method is required".to_string(),
            None::<()>,
        )
    })?;

    let params = scheduled_tx.get("params").unwrap_or(&Value::Null).clone();

    match method {
        "createTopic" => {
            let tx = crate::methods::topic::build_topic_create_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "submitMessage" => {
            let tx = crate::methods::topic::build_topic_message_submit_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "createAccount" => {
            let tx = crate::methods::account::build_account_create_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "transferCrypto" => {
            let tx = crate::server::build_transfer_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "approveAllowance" => {
            let tx = crate::methods::account::build_allowance_approve_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "mintToken" => {
            let tx = crate::methods::token::build_token_mint_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        "burnToken" => {
            let tx = crate::methods::token::build_token_burn_tx_from_value(&params)?;
            schedule_tx.scheduled_transaction(tx);
            Ok(())
        }
        _ => Err(ErrorObject::owned(
            INTERNAL_ERROR_CODE,
            format!("Unsupported scheduled transaction method: {method}"),
            None::<()>,
        )),
    }
}
