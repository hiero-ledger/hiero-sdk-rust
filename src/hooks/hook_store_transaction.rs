use hiero_sdk_proto::services;
use hiero_sdk_proto::services::smart_contract_service_client::SmartContractServiceClient;
use tonic::transport::Channel;

use crate::hooks::{
    EvmHookStorageUpdate,
    HookId,
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
pub type HookStoreTransaction = Transaction<HookStoreTransactionData>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookStoreTransactionData {
    /// The hook ID to store data for.
    hook_id: Option<HookId>,
    /// The storage updates to apply.
    storage_updates: Vec<EvmHookStorageUpdate>,
}

impl Default for HookStoreTransactionData {
    fn default() -> Self {
        Self { hook_id: None, storage_updates: Vec::new() }
    }
}

impl HookStoreTransaction {
    /// Set the hook ID.
    pub fn set_hook_id(&mut self, hook_id: HookId) -> &mut Self {
        self.data_mut().hook_id = Some(hook_id);
        self
    }

    /// Set the storage updates.
    pub fn set_storage_updates(&mut self, storage_updates: Vec<EvmHookStorageUpdate>) -> &mut Self {
        self.data_mut().storage_updates = storage_updates;
        self
    }

    /// Add a storage update.
    pub fn add_storage_update(&mut self, storage_update: EvmHookStorageUpdate) -> &mut Self {
        self.data_mut().storage_updates.push(storage_update);
        self
    }

    /// Get the hook ID.
    pub fn get_hook_id(&self) -> Option<&HookId> {
        self.data().hook_id.as_ref()
    }

    /// Get the storage updates.
    pub fn get_storage_updates(&self) -> &[EvmHookStorageUpdate] {
        &self.data().storage_updates
    }
}

impl HookStoreTransactionData {
    /// Create a new `HookStoreTransactionData`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TransactionData for HookStoreTransactionData {
    // 2 habrs are the default max transaction fee for most transaction acrooss the SDK
    fn default_max_transaction_fee(&self) -> crate::Hbar {
        crate::Hbar::new(2)
    }
}

impl ValidateChecksums for HookStoreTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), crate::Error> {
        if let Some(hook_id) = &self.hook_id {
            hook_id.entity_id.validate_checksums(ledger_id)?;
        }
        Ok(())
    }
}

impl TransactionExecute for HookStoreTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { SmartContractServiceClient::new(channel).hook_store(request).await })
    }
}

impl ToTransactionDataProtobuf for HookStoreTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();
        services::transaction_body::Data::HookStore(self.to_protobuf())
    }
}

impl crate::protobuf::ToProtobuf for HookStoreTransactionData {
    type Protobuf = services::HookStoreTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::HookStoreTransactionBody {
            hook_id: self.hook_id.as_ref().map(|id| id.to_protobuf()),
            storage_updates: self
                .storage_updates
                .iter()
                .map(|update| update.to_protobuf())
                .collect(),
        }
    }
}

impl crate::protobuf::FromProtobuf<services::HookStoreTransactionBody>
    for HookStoreTransactionData
{
    fn from_protobuf(pb: services::HookStoreTransactionBody) -> crate::Result<Self> {
        let hook_id = pb.hook_id.map(HookId::from_protobuf).transpose()?;

        let storage_updates = pb
            .storage_updates
            .into_iter()
            .map(EvmHookStorageUpdate::from_protobuf)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { hook_id, storage_updates })
    }
}

impl From<HookStoreTransactionData> for crate::transaction::AnyTransactionData {
    fn from(transaction: HookStoreTransactionData) -> Self {
        Self::HookStore(transaction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::{
        EvmHookStorageSlot,
        EvmHookStorageUpdate,
        HookId,
    };

    #[test]
    fn test_hook_store_transaction_creation() {
        let hook_id = HookId::new(None, 123);
        let storage_slot = EvmHookStorageSlot::new(vec![1, 2, 3], vec![4, 5, 6]);
        let storage_update = EvmHookStorageUpdate::StorageSlot(storage_slot);

        let mut transaction = HookStoreTransaction::new();
        transaction.set_hook_id(hook_id.clone()).add_storage_update(storage_update);

        assert_eq!(transaction.get_hook_id(), Some(&hook_id));
        assert_eq!(transaction.get_storage_updates().len(), 1);
    }

    #[test]
    fn test_hook_store_transaction_default() {
        let transaction = HookStoreTransaction::new();
        assert_eq!(transaction.get_hook_id(), None);
        assert_eq!(transaction.get_storage_updates().len(), 0);
    }
}
