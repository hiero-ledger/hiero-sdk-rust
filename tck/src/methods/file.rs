use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    Client,
    FileAppendTransaction,
    FileContentsQuery,
    FileCreateTransaction,
    FileDeleteTransaction,
    FileId,
    FileInfoQuery,
    FileUpdateTransaction,
    Hbar,
};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;
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
};
use crate::responses::{
    FileContentsResponse,
    FileInfoResponse,
    FileResponse,
};

#[rpc(server, client)]
pub trait FileRpc {
    #[method(name = "createFile")]
    async fn create_file(
        &self,
        keys: Option<Vec<String>>,
        contents: Option<String>,
        expiration_time: Option<String>,
        memo: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<FileResponse, ErrorObjectOwned>;

    #[method(name = "appendFile")]
    async fn append_file(
        &self,
        file_id: Option<String>,
        contents: Option<String>,
        max_chunks: Option<i64>,
        chunk_size: Option<i64>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<FileResponse, ErrorObjectOwned>;

    #[method(name = "updateFile")]
    async fn update_file(
        &self,
        file_id: Option<String>,
        keys: Option<Vec<String>>,
        contents: Option<String>,
        expiration_time: Option<String>,
        memo: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<FileResponse, ErrorObjectOwned>;

    #[method(name = "deleteFile")]
    async fn delete_file(
        &self,
        file_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<FileResponse, ErrorObjectOwned>;

    #[method(name = "getFileInfo")]
    async fn get_file_info(
        &self,
        file_id: Option<String>,
        query_payment: Option<String>,
        max_query_payment: Option<String>,
        get_cost: Option<bool>,
    ) -> Result<FileInfoResponse, ErrorObjectOwned>;

    #[method(name = "getFileContents")]
    async fn get_file_contents(
        &self,
        file_id: Option<String>,
        query_payment: Option<String>,
        max_query_payment: Option<String>,
    ) -> Result<FileContentsResponse, ErrorObjectOwned>;
}

pub async fn create_file(
    client: &Client,
    keys: Option<Vec<String>>,
    contents: Option<String>,
    expiration_time: Option<String>,
    memo: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<FileResponse, ErrorObjectOwned> {
    let mut tx = FileCreateTransaction::new();

    if let Some(keys) = keys {
        let parsed_keys: Result<Vec<_>, _> = keys.iter().map(|k| get_hedera_key(k)).collect();
        let parsed_keys = parsed_keys?;
        tx.keys(parsed_keys);
    }

    if let Some(contents) = contents {
        tx.contents(contents.as_bytes());
    }

    if let Some(expiration_time) = expiration_time {
        let timestamp = expiration_time
            .parse::<i64>()
            .map_err(|e| mock_consensus_error("AUTORENEW_DURATION_NOT_IN_RANGE", &e.to_string()))?;
        let offset = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| mock_consensus_error("AUTORENEW_DURATION_NOT_IN_RANGE", &e.to_string()))?;
        tx.expiration_time(offset);
    }

    if let Some(memo) = memo {
        tx.file_memo(memo);
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(FileResponse {
        file_id: tx_receipt.file_id.map(|id| id.to_string()),
        status: tx_receipt.status.as_str_name().to_string(),
    })
}

pub async fn append_file(
    client: &Client,
    file_id: Option<String>,
    contents: Option<String>,
    max_chunks: Option<i64>,
    chunk_size: Option<i64>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<FileResponse, ErrorObjectOwned> {
    let mut tx = FileAppendTransaction::new();

    if let Some(file_id) = file_id {
        tx.file_id(FileId::from_str(&file_id).map_err(internal_error)?);
    }

    if let Some(contents) = contents {
        tx.contents(contents.as_bytes());
    }

    if let Some(max_chunks) = max_chunks {
        if max_chunks < 0 {
            return Err(internal_error("Max chunks cannot be negative"));
        }
        tx.max_chunks(max_chunks as usize);
    }

    if let Some(chunk_size) = chunk_size {
        if chunk_size < 0 {
            return Err(internal_error("Chunk size cannot be negative"));
        }
        if chunk_size == 0 {
            return Err(internal_error("Chunk size must be greater than zero"));
        }
        tx.chunk_size(chunk_size as usize);
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(FileResponse { file_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn update_file(
    client: &Client,
    file_id: Option<String>,
    keys: Option<Vec<String>>,
    contents: Option<String>,
    expiration_time: Option<String>,
    memo: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<FileResponse, ErrorObjectOwned> {
    let mut tx = FileUpdateTransaction::new();

    if let Some(file_id) = file_id {
        tx.file_id(FileId::from_str(&file_id).map_err(internal_error)?);
    }

    if let Some(keys) = keys {
        let parsed_keys: Result<Vec<_>, _> = keys.iter().map(|k| get_hedera_key(k)).collect();
        let parsed_keys = parsed_keys?;
        tx.keys(parsed_keys);
    }

    if let Some(contents) = contents {
        tx.contents(contents.as_bytes().to_vec());
    }

    if let Some(expiration_time) = expiration_time {
        let timestamp = expiration_time
            .parse::<i64>()
            .map_err(|e| mock_consensus_error("AUTORENEW_DURATION_NOT_IN_RANGE", &e.to_string()))?;
        let offset = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| mock_consensus_error("AUTORENEW_DURATION_NOT_IN_RANGE", &e.to_string()))?;
        tx.expiration_time(offset);
    }

    if let Some(memo) = memo {
        tx.file_memo(memo);
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(FileResponse { file_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn delete_file(
    client: &Client,
    file_id: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<FileResponse, ErrorObjectOwned> {
    let mut tx = FileDeleteTransaction::new();

    if let Some(file_id) = file_id {
        tx.file_id(FileId::from_str(&file_id).map_err(internal_error)?);
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(FileResponse { file_id: None, status: tx_receipt.status.as_str_name().to_string() })
}

pub async fn get_file_info(
    client: &Client,
    file_id: Option<String>,
    query_payment: Option<String>,
    max_query_payment: Option<String>,
    get_cost: Option<bool>,
) -> Result<FileInfoResponse, ErrorObjectOwned> {
    let mut query = FileInfoQuery::new();

    if let Some(file_id) = file_id {
        query.file_id(FileId::from_str(&file_id).map_err(internal_error)?);
    }

    if let Some(query_payment) = query_payment {
        let amount = query_payment.parse::<i64>().map_err(internal_error)?;
        query.payment_amount(Hbar::from_tinybars(amount));
    }

    if let Some(max_query_payment) = max_query_payment {
        let amount = max_query_payment.parse::<i64>().map_err(internal_error)?;
        query.max_payment_amount(Hbar::from_tinybars(amount));
    }

    if get_cost.unwrap_or(false) {
        let cost = query.get_cost(client).await.map_err(from_hedera_error)?;
        return Ok(FileInfoResponse {
            file_id: None,
            size: None,
            expiration_time: None,
            is_deleted: None,
            keys: None,
            memo: None,
            ledger_id: None,
            cost: Some(cost.to_tinybars().to_string()),
        });
    }

    let response = query.execute(client).await.map_err(from_hedera_error)?;

    Ok(FileInfoResponse {
        file_id: Some(response.file_id.to_string()),
        size: Some(response.size.to_string()),
        expiration_time: response.expiration_time.map(|t| t.unix_timestamp().to_string()),
        is_deleted: Some(response.is_deleted),
        keys: Some(
            response
                .keys
                .keys
                .iter()
                .map(|key| match key {
                    hiero_sdk::Key::Single(pk) => pk.to_string_der(),
                    _ => hex::encode(key.to_bytes()),
                })
                .collect(),
        ),
        memo: Some(response.file_memo),
        ledger_id: Some(response.ledger_id.to_string()),
        cost: None,
    })
}

pub async fn get_file_contents(
    client: &Client,
    file_id: Option<String>,
    query_payment: Option<String>,
    max_query_payment: Option<String>,
) -> Result<FileContentsResponse, ErrorObjectOwned> {
    let mut query = FileContentsQuery::new();

    if let Some(file_id) = file_id {
        query.file_id(FileId::from_str(&file_id).map_err(internal_error)?);
    }

    if let Some(query_payment) = query_payment {
        let amount = query_payment.parse::<i64>().map_err(internal_error)?;
        query.payment_amount(Hbar::from_tinybars(amount));
    }

    if let Some(max_query_payment) = max_query_payment {
        let amount = max_query_payment.parse::<i64>().map_err(internal_error)?;
        query.max_payment_amount(Hbar::from_tinybars(amount));
    }

    let response = query.execute(client).await.map_err(from_hedera_error)?;

    // Convert Vec<u8> to String, handling potential UTF-8 errors gracefully
    let contents = String::from_utf8_lossy(&response.contents).to_string();

    Ok(FileContentsResponse { contents })
}
