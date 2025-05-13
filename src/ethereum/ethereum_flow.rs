// SPDX-License-Identifier: Apache-2.0

use std::mem;

use super::ethereum_data::EthereumData;
use crate::{
    Client,
    EthereumTransaction,
    FileAppendTransaction,
    FileCreateTransaction,
    FileId,
    Hbar,
    TransactionResponse,
};

/// Flow for executing ethereum transactions.
#[derive(Default, Debug)]
pub struct EthereumFlow {
    ethereum_data: Option<EthereumData>,
    max_gas_allowance: Option<Hbar>,
}

impl EthereumFlow {
    const MAX_ETHEREUM_DATA_SIZE: usize = 5120;

    /// Create a new `EthereumFlow` ready for configuartion.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the raw Ethereum transaction (RLP encoded type 0, 1, and 2).
    #[must_use]
    pub fn get_ethereum_data(&self) -> Option<&EthereumData> {
        self.ethereum_data.as_ref()
    }

    /// Sets the raw Ethereum transaction data (RLP encoded type 0, 1, and 2).
    ///
    /// # Errors
    /// - [`Error::BasicParse`](crate::Error::BasicParse) if the given `data` cannot be parsed as [`EthereumData`].
    pub fn ethereum_data(&mut self, data: &[u8]) -> crate::Result<&mut Self> {
        self.ethereum_data = Some(EthereumData::from_bytes(data)?);

        Ok(self)
    }

    /// Returns the maximum amount that the payer of the hedera transaction is willing to pay to complete the transaction.
    #[must_use]
    pub fn get_max_gas_allowance(&self) -> Option<Hbar> {
        self.max_gas_allowance
    }

    /// Sets the maximum amount that the payer of the ethereum transaction is willing to pay to complete the transaction.
    pub fn max_gas_allowance(&mut self, hbar: Hbar) -> &mut Self {
        self.max_gas_allowance = Some(hbar);

        self
    }

    /// Generates the required transactions and executes them all.
    pub async fn execute(&self, client: &Client) -> crate::Result<TransactionResponse> {
        self.execute_with_optional_timeout(client, None).await
    }

    /// Generates the required transactions and executes them all.
    pub async fn execute_with_timeout(
        &self,
        client: &Client,
        timeout_per_transaction: std::time::Duration,
    ) -> crate::Result<TransactionResponse> {
        self.execute_with_optional_timeout(client, Some(timeout_per_transaction)).await
    }

    async fn execute_with_optional_timeout(
        &self,
        client: &Client,
        timeout_per_transaction: Option<std::time::Duration>,
    ) -> crate::Result<TransactionResponse> {
        let mut ethereum_data = self
            .ethereum_data
            .clone()
            .expect("Must set ethereum data before calling execute on ethereum flow");

        let ethereum_data_bytes = ethereum_data.to_bytes();

        let mut ethereum_transaction = EthereumTransaction::new();

        if let Some(allowance) = self.max_gas_allowance {
            ethereum_transaction.max_gas_allowance_hbar(allowance);
        }

        if ethereum_data_bytes.len() <= Self::MAX_ETHEREUM_DATA_SIZE {
            return ethereum_transaction
                .ethereum_data(ethereum_data_bytes)
                .execute_with_optional_timeout(client, timeout_per_transaction)
                .await;
        }

        let call_data = mem::take(ethereum_data.call_data_mut());
        let file_id = create_file(client, call_data, timeout_per_transaction).await?;
        let ethereum_data_bytes = ethereum_data.to_bytes();
        ethereum_transaction.call_data_file_id(file_id).ethereum_data(ethereum_data_bytes);

        ethereum_transaction.execute_with_optional_timeout(client, timeout_per_transaction).await
    }
}

fn split_call_data(call_data: Vec<u8>) -> (Vec<u8>, Option<Vec<u8>>) {
    const FILE_APPEND_DEFAULT_CHUNK_SIZE: usize = 4096;

    if call_data.len() <= FILE_APPEND_DEFAULT_CHUNK_SIZE {
        return (call_data, None);
    }

    let mut file_create_call_data = call_data;
    let file_append_call_data = file_create_call_data.split_off(FILE_APPEND_DEFAULT_CHUNK_SIZE);

    // note: this uses `subdata` because `Data` is it's own subsequence...
    // It's weirdly written such that the `fileAppendData` wouldn't start at index 0
    // even though that's literally what you'd expect.
    (file_create_call_data, Some(file_append_call_data))
}

async fn create_file(
    client: &Client,
    call_data: Vec<u8>,
    timeout_per_transaction: Option<std::time::Duration>,
) -> crate::Result<FileId> {
    let (file_create_data, file_append_data) = split_call_data(call_data);

    let file_id = FileCreateTransaction::new()
        .contents(file_create_data)
        .execute_with_optional_timeout(client, timeout_per_transaction)
        .await?
        .get_receipt_query()
        .execute_with_optional_timeout(client, timeout_per_transaction)
        .await?
        .file_id
        .expect("Creating a file means there's a file ID");

    if let Some(file_append_data) = file_append_data {
        FileAppendTransaction::new()
            .file_id(file_id)
            .contents(file_append_data)
            .execute_all_with_optional_timeout(client, timeout_per_transaction)
            .await?;
    }

    Ok(file_id)
}
