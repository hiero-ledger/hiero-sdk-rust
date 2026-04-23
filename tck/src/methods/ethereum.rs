use std::collections::HashMap;
use std::str::FromStr;

use hiero_sdk::{
    Client,
    EthereumTransaction,
    FileId,
    Hbar,
};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;
use serde_json::Value;

use crate::common::internal_error;
use crate::errors::from_hedera_error;
use crate::helpers::fill_common_transaction_params;
use crate::responses::EthereumResponse;

#[rpc(server, client)]
pub trait EthereumRpc {
    #[method(name = "createEthereumTransaction")]
    async fn create_ethereum_transaction(
        &self,
        ethereum_data: Option<String>,
        call_data_file_id: Option<String>,
        max_gas_allowance: Option<String>,
        common_transaction_params: Option<HashMap<String, Value>>,
    ) -> Result<EthereumResponse, ErrorObjectOwned>;
}

pub async fn create_ethereum_transaction(
    client: &Client,
    ethereum_data: Option<String>,
    call_data_file_id: Option<String>,
    max_gas_allowance: Option<String>,
    common_transaction_params: Option<HashMap<String, Value>>,
) -> Result<EthereumResponse, ErrorObjectOwned> {
    let mut tx = EthereumTransaction::new();

    if let Some(ethereum_data) = ethereum_data {
        // Decode hex string to bytes
        let bytes = hex::decode(&ethereum_data)
            .map_err(|e| internal_error(format!("Failed to decode ethereum data hex: {e}")))?;
        tx.ethereum_data(bytes);
    }

    if let Some(call_data_file_id) = call_data_file_id {
        tx.call_data_file_id(FileId::from_str(&call_data_file_id).map_err(internal_error)?);
    }

    if let Some(max_gas_allowance) = max_gas_allowance {
        let amount = max_gas_allowance.parse::<i64>().map_err(internal_error)?;
        tx.max_gas_allowance_hbar(Hbar::from_tinybars(amount));
    }

    if let Some(params) = common_transaction_params {
        fill_common_transaction_params(&mut tx, &params, client);
    }

    let tx_response = tx.execute(client).await.map_err(|e| from_hedera_error(e))?;
    let tx_receipt = tx_response.get_receipt(client).await.map_err(|e| from_hedera_error(e))?;

    Ok(EthereumResponse {
        status: tx_receipt.status.as_str_name().to_string(),
        contract_id: tx_receipt.contract_id.map(|id| id.to_string()),
    })
}
