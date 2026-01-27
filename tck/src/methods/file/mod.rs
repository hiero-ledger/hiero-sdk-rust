use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    Client,
    FileAppendTransaction,
    FileCreateTransaction,
    FileDeleteTransaction,
    FileId,
};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use serde_json::Value;
use time::OffsetDateTime;

use crate::errors::from_hedera_error;
use crate::helpers::{
    fill_common_transaction_params,
    get_hedera_key,
};
use crate::responses::FileResponse;

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
        max_chunks: Option<usize>,
        chunk_size: Option<usize>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<FileResponse, ErrorObjectOwned>;

    #[method(name = "deleteFile")]
    async fn delete_file(
        &self,
        file_id: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<FileResponse, ErrorObjectOwned>;
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
        if !keys.is_empty() {
            let parsed_keys: Result<Vec<_>, _> = keys.iter().map(|k| get_hedera_key(k)).collect();
            let parsed_keys = parsed_keys?;
            tx.keys(parsed_keys);
        }
    }

    if let Some(contents) = contents {
        tx.contents(contents.as_bytes());
    }

    if let Some(expiration_time) = expiration_time {
        let timestamp = expiration_time
            .parse::<i64>()
            .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
        let offset = OffsetDateTime::from_unix_timestamp(timestamp)
            .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?;
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
    max_chunks: Option<usize>,
    chunk_size: Option<usize>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<FileResponse, ErrorObjectOwned> {
    let mut tx = FileAppendTransaction::new();

    if let Some(file_id) = file_id {
        tx.file_id(
            FileId::from_str(&file_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(contents) = contents {
        tx.contents(contents.as_bytes());
    }

    if let Some(max_chunks) = max_chunks {
        tx.max_chunks(max_chunks);
    }

    if let Some(chunk_size) = chunk_size {
        if chunk_size == 0 {
            return Err(ErrorObject::owned(
                INTERNAL_ERROR_CODE,
                "Chunk size must be greater than zero".to_string(),
                None::<()>,
            ));
        }
        tx.chunk_size(chunk_size);
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
        tx.file_id(
            FileId::from_str(&file_id)
                .map_err(|e| ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>))?,
        );
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(FileResponse { file_id: None, status: tx_receipt.status.as_str_name().to_string() })
}
