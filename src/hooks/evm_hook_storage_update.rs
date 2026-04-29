use hiero_sdk_proto::services;

use crate::hooks::EvmHookStorageSlot;
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// A lambda storage update containing either a storage slot or mapping entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmHookStorageUpdate {
    StorageSlot(EvmHookStorageSlot),
    MappingEntries(EvmHookMappingEntries),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmHookMappingEntries {
    pub mapping_slot: Vec<u8>,
    pub entries: Vec<EvmHookMappingEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmHookMappingEntry {
    pub key: Option<Vec<u8>>,
    pub value: Option<Vec<u8>>,
    pub preimage: Option<Vec<u8>>,
}

impl EvmHookStorageUpdate {}

impl EvmHookMappingEntries {
    pub fn new(mapping_slot: Vec<u8>, entries: Vec<EvmHookMappingEntry>) -> Self {
        Self { mapping_slot, entries }
    }
}

impl EvmHookMappingEntry {
    pub fn new(key: Option<Vec<u8>>, value: Option<Vec<u8>>) -> Self {
        Self { key, value, preimage: None }
    }

    pub fn set_key(&mut self, key: Vec<u8>) -> &mut Self {
        self.key = Some(key);
        self.preimage = None; // Clear preimage since they're mutually exclusive
        self
    }

    pub fn set_value(&mut self, value: Vec<u8>) -> &mut Self {
        self.value = Some(value);
        self
    }

    pub fn set_preimage(&mut self, preimage: Vec<u8>) -> &mut Self {
        self.preimage = Some(preimage);
        self.key = None; // Clear key since they're mutually exclusive
        self
    }
}

impl ToProtobuf for EvmHookStorageUpdate {
    type Protobuf = services::EvmHookStorageUpdate;

    fn to_protobuf(&self) -> Self::Protobuf {
        match self {
            Self::StorageSlot(slot) => services::EvmHookStorageUpdate {
                update: Some(services::evm_hook_storage_update::Update::StorageSlot(
                    slot.to_protobuf(),
                )),
            },
            Self::MappingEntries(entries) => services::EvmHookStorageUpdate {
                update: Some(services::evm_hook_storage_update::Update::MappingEntries(
                    entries.to_protobuf(),
                )),
            },
        }
    }
}

impl FromProtobuf<services::EvmHookStorageUpdate> for EvmHookStorageUpdate {
    fn from_protobuf(pb: services::EvmHookStorageUpdate) -> crate::Result<Self> {
        match pb.update {
            Some(services::evm_hook_storage_update::Update::StorageSlot(slot)) => {
                Ok(Self::StorageSlot(EvmHookStorageSlot::from_protobuf(slot)?))
            }
            Some(services::evm_hook_storage_update::Update::MappingEntries(entries)) => {
                Ok(Self::MappingEntries(EvmHookMappingEntries::from_protobuf(entries)?))
            }
            None => Err(crate::Error::basic_parse(
                "EvmHookStorageUpdate must have either storage_slot or mapping_entries",
            )),
        }
    }
}

impl ToProtobuf for EvmHookMappingEntries {
    type Protobuf = services::EvmHookMappingEntries;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::EvmHookMappingEntries {
            mapping_slot: self.mapping_slot.clone(),
            entries: self.entries.iter().map(|entry| entry.to_protobuf()).collect(),
        }
    }
}

impl FromProtobuf<services::EvmHookMappingEntries> for EvmHookMappingEntries {
    fn from_protobuf(pb: services::EvmHookMappingEntries) -> crate::Result<Self> {
        let entries = pb
            .entries
            .into_iter()
            .map(EvmHookMappingEntry::from_protobuf)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { mapping_slot: pb.mapping_slot, entries })
    }
}

impl ToProtobuf for EvmHookMappingEntry {
    type Protobuf = services::EvmHookMappingEntry;

    fn to_protobuf(&self) -> Self::Protobuf {
        let entry_key = if let Some(key) = &self.key {
            Some(services::evm_hook_mapping_entry::EntryKey::Key(key.clone()))
        } else if let Some(preimage) = &self.preimage {
            Some(services::evm_hook_mapping_entry::EntryKey::Preimage(preimage.clone()))
        } else {
            None
        };

        services::EvmHookMappingEntry { entry_key, value: self.value.clone().unwrap_or_default() }
    }
}

impl FromProtobuf<services::EvmHookMappingEntry> for EvmHookMappingEntry {
    fn from_protobuf(pb: services::EvmHookMappingEntry) -> crate::Result<Self> {
        let (key, preimage) = match pb.entry_key {
            Some(services::evm_hook_mapping_entry::EntryKey::Key(k)) => (Some(k), None),
            Some(services::evm_hook_mapping_entry::EntryKey::Preimage(p)) => (None, Some(p)),
            None => (None, None),
        };

        Ok(Self { key, value: if pb.value.is_empty() { None } else { Some(pb.value) }, preimage })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_hook_mapping_entry_creation() {
        let entry = EvmHookMappingEntry::new(Some(vec![1, 2, 3]), Some(vec![4, 5, 6]));
        assert_eq!(entry.key, Some(vec![1, 2, 3]));
        assert_eq!(entry.value, Some(vec![4, 5, 6]));
        assert_eq!(entry.preimage, None);
    }

    #[test]
    fn test_evm_hook_mapping_entry_with_preimage() {
        let mut entry = EvmHookMappingEntry::new(None, Some(vec![10, 11, 12]));
        entry.set_preimage(vec![7, 8, 9]);
        assert_eq!(entry.key, None);
        assert_eq!(entry.preimage, Some(vec![7, 8, 9]));
        assert_eq!(entry.value, Some(vec![10, 11, 12]));
    }

    #[test]
    fn test_evm_hook_mapping_entry_setters() {
        let mut entry = EvmHookMappingEntry::new(None, None);
        entry.set_key(vec![7, 8, 9]).set_value(vec![10, 11, 12]);

        assert_eq!(entry.key, Some(vec![7, 8, 9]));
        assert_eq!(entry.value, Some(vec![10, 11, 12]));
        assert_eq!(entry.preimage, None);
    }

    #[test]
    fn test_evm_hook_mapping_entry_key_preimage_mutual_exclusion() {
        let mut entry = EvmHookMappingEntry::new(Some(vec![1, 2, 3]), None);
        assert_eq!(entry.key, Some(vec![1, 2, 3]));
        assert_eq!(entry.preimage, None);

        // Setting preimage should clear key
        entry.set_preimage(vec![4, 5, 6]);
        assert_eq!(entry.key, None);
        assert_eq!(entry.preimage, Some(vec![4, 5, 6]));

        // Setting key should clear preimage
        entry.set_key(vec![7, 8, 9]);
        assert_eq!(entry.key, Some(vec![7, 8, 9]));
        assert_eq!(entry.preimage, None);
    }

    #[test]
    fn test_evm_hook_mapping_entry_protobuf_roundtrip() {
        let original = EvmHookMappingEntry::new(Some(vec![1, 2, 3]), Some(vec![4, 5, 6]));
        let protobuf = original.to_protobuf();
        let reconstructed = EvmHookMappingEntry::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
