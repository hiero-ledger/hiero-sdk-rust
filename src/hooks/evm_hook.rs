use hiero_sdk_proto::services;

use crate::hooks::{
    EvmHookSpec,
    EvmHookStorageUpdate,
};
use crate::{
    FromProtobuf,
    ToProtobuf,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmHook {
    pub spec: EvmHookSpec,
    pub storage_updates: Vec<EvmHookStorageUpdate>,
}

impl EvmHook {
    pub fn new(spec: EvmHookSpec, storage_updates: Vec<EvmHookStorageUpdate>) -> Self {
        Self { spec, storage_updates }
    }

    pub fn set_storage_updates(&mut self, storage_updates: Vec<EvmHookStorageUpdate>) -> &mut Self {
        self.storage_updates = storage_updates;
        self
    }

    pub fn add_storage_update(&mut self, storage_update: EvmHookStorageUpdate) -> &mut Self {
        self.storage_updates.push(storage_update);
        self
    }
}

impl ToProtobuf for EvmHook {
    type Protobuf = services::EvmHook;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::EvmHook {
            spec: Some(self.spec.to_protobuf()),
            storage_updates: self
                .storage_updates
                .iter()
                .map(|update| update.to_protobuf())
                .collect(),
        }
    }
}

impl FromProtobuf<services::EvmHook> for EvmHook {
    fn from_protobuf(pb: services::EvmHook) -> crate::Result<Self> {
        let spec = pb
            .spec
            .map(EvmHookSpec::from_protobuf)
            .transpose()?
            .unwrap_or_else(|| EvmHookSpec::new(None));

        let storage_updates = pb
            .storage_updates
            .into_iter()
            .map(EvmHookStorageUpdate::from_protobuf)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { spec, storage_updates })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::ContractId;
    use crate::hooks::EvmHookStorageSlot;

    #[test]
    fn test_evm_hook_creation() {
        let contract_id = ContractId::new(0, 0, 123);
        let spec = EvmHookSpec::new(Some(contract_id));
        let storage_slot = EvmHookStorageSlot::new(vec![1, 2, 3], vec![4, 5, 6]);
        let storage_update = EvmHookStorageUpdate::StorageSlot(storage_slot);
        let storage_updates = vec![storage_update];

        let hook = EvmHook::new(spec.clone(), storage_updates.clone());

        assert_eq!(hook.spec, spec);
        assert_eq!(hook.storage_updates, storage_updates);
    }

    #[test]
    fn test_evm_hook_with_spec_only() {
        let contract_id = ContractId::new(0, 0, 456);
        let spec = EvmHookSpec::new(Some(contract_id));
        let hook = EvmHook::new(spec.clone(), vec![]);

        assert_eq!(hook.spec, spec);
        assert_eq!(hook.storage_updates.len(), 0);
    }

    #[test]
    fn test_evm_hook_setters() {
        let contract_id = ContractId::new(0, 0, 789);
        let spec = EvmHookSpec::new(Some(contract_id));
        let mut hook = EvmHook::new(spec, vec![]);

        let storage_slot = EvmHookStorageSlot::new(vec![1, 2, 3], vec![4, 5, 6]);
        let storage_update = EvmHookStorageUpdate::StorageSlot(storage_slot);
        let storage_updates = vec![storage_update.clone()];

        hook.set_storage_updates(storage_updates.clone());
        assert_eq!(hook.storage_updates, storage_updates);

        let another_slot = EvmHookStorageSlot::new(vec![7, 8, 9], vec![10, 11, 12]);
        let another_update = EvmHookStorageUpdate::StorageSlot(another_slot);
        hook.add_storage_update(another_update);

        assert_eq!(hook.storage_updates.len(), 2);
    }

    #[test]
    fn test_evm_hook_protobuf_roundtrip() {
        let contract_id = ContractId::new(0, 0, 111);
        let spec = EvmHookSpec::new(Some(contract_id));
        let storage_slot = EvmHookStorageSlot::new(vec![1, 2, 3], vec![4, 5, 6]);
        let storage_update = EvmHookStorageUpdate::StorageSlot(storage_slot);
        let original = EvmHook::new(spec, vec![storage_update]);

        let protobuf = original.to_protobuf();
        let reconstructed = EvmHook::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
