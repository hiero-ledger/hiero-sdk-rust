use hedera_proto::services;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use tonic::transport::Channel;

use crate::hooks::{
    HookId,
    LambdaStorageUpdate,
};
use crate::ledger_id::RefLedgerId;
use crate::protobuf::ToProtobuf;
use crate::transaction::{
    ChunkInfo,
    ToTransactionDataProtobuf,
    Transaction,
    TransactionData,
    TransactionExecute,
};
use crate::{
    BoxGrpcFuture,
    ValidateChecksums,
};

/// A transaction to store lambda data in hook storage.
pub type LambdaSStoreTransaction = Transaction<LambdaSStoreTransactionData>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaSStoreTransactionData {
    /// The hook ID to store data for.
    hook_id: Option<HookId>,
    /// The storage updates to apply.
    storage_updates: Vec<LambdaStorageUpdate>,
}

impl Default for LambdaSStoreTransactionData {
    fn default() -> Self {
        Self { hook_id: None, storage_updates: Vec::new() }
    }
}

impl LambdaSStoreTransaction {
    /// Set the hook ID.
    pub fn set_hook_id(&mut self, hook_id: HookId) -> &mut Self {
        self.data_mut().hook_id = Some(hook_id);
        self
    }

    /// Set the storage updates.
    pub fn set_storage_updates(&mut self, storage_updates: Vec<LambdaStorageUpdate>) -> &mut Self {
        self.data_mut().storage_updates = storage_updates;
        self
    }

    /// Add a storage update.
    pub fn add_storage_update(&mut self, storage_update: LambdaStorageUpdate) -> &mut Self {
        self.data_mut().storage_updates.push(storage_update);
        self
    }

    /// Get the hook ID.
    pub fn get_hook_id(&self) -> Option<&HookId> {
        self.data().hook_id.as_ref()
    }

    /// Get the storage updates.
    pub fn get_storage_updates(&self) -> &[LambdaStorageUpdate] {
        &self.data().storage_updates
    }
}

impl LambdaSStoreTransactionData {
    /// Create a new `LambdaSStoreTransactionData`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TransactionData for LambdaSStoreTransactionData {
    fn default_max_transaction_fee(&self) -> crate::Hbar {
        crate::Hbar::new(2)
    }
}

impl ValidateChecksums for LambdaSStoreTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), crate::Error> {
        if let Some(hook_id) = &self.hook_id {
            hook_id.entity_id.validate_checksums(ledger_id)?;
        }
        Ok(())
    }
}

impl TransactionExecute for LambdaSStoreTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { SmartContractServiceClient::new(channel).lambda_s_store(request).await })
    }
}

impl ToTransactionDataProtobuf for LambdaSStoreTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();
        services::transaction_body::Data::LambdaSstore(self.to_protobuf())
    }
}

impl crate::protobuf::ToProtobuf for LambdaSStoreTransactionData {
    type Protobuf = services::LambdaSStoreTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::LambdaSStoreTransactionBody {
            hook_id: self.hook_id.as_ref().map(|id| id.to_protobuf()),
            storage_updates: self
                .storage_updates
                .iter()
                .map(|update| update.to_protobuf())
                .collect(),
        }
    }
}

impl crate::protobuf::FromProtobuf<services::LambdaSStoreTransactionBody>
    for LambdaSStoreTransactionData
{
    fn from_protobuf(pb: services::LambdaSStoreTransactionBody) -> crate::Result<Self> {
        let hook_id = pb.hook_id.map(HookId::from_protobuf).transpose()?;

        let storage_updates = pb
            .storage_updates
            .into_iter()
            .map(LambdaStorageUpdate::from_protobuf)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { hook_id, storage_updates })
    }
}

impl From<LambdaSStoreTransactionData> for crate::transaction::AnyTransactionData {
    fn from(transaction: LambdaSStoreTransactionData) -> Self {
        Self::LambdaSStore(transaction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::{
        HookId,
        LambdaStorageSlot,
        LambdaStorageUpdate,
    };

    #[test]
    fn test_lambda_s_store_transaction_creation() {
        let hook_id = HookId::new(None, 123);
        let storage_slot = LambdaStorageSlot::new(vec![1, 2, 3], vec![4, 5, 6]);
        let storage_update = LambdaStorageUpdate::StorageSlot(storage_slot);

        let mut transaction = LambdaSStoreTransaction::new();
        transaction.set_hook_id(hook_id.clone()).add_storage_update(storage_update);

        assert_eq!(transaction.get_hook_id(), Some(&hook_id));
        assert_eq!(transaction.get_storage_updates().len(), 1);
    }

    #[test]
    fn test_lambda_s_store_transaction_default() {
        let transaction = LambdaSStoreTransaction::new();
        assert_eq!(transaction.get_hook_id(), None);
        assert_eq!(transaction.get_storage_updates().len(), 0);
    }
}
