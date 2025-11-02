use hedera_proto::services;

use crate::hooks::LambdaStorageSlot;
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// A lambda storage update containing either a storage slot or mapping entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LambdaStorageUpdate {
    StorageSlot(LambdaStorageSlot),
    MappingEntries(LambdaMappingEntries),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaMappingEntries {
    pub mapping_slot: Vec<u8>,
    pub entries: Vec<LambdaMappingEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaMappingEntry {
    pub key: Option<Vec<u8>>,
    pub value: Option<Vec<u8>>,
    pub preimage: Option<Vec<u8>>,
}

impl LambdaStorageUpdate {}

impl LambdaMappingEntries {
    pub fn new(mapping_slot: Vec<u8>, entries: Vec<LambdaMappingEntry>) -> Self {
        Self { mapping_slot, entries }
    }
}

impl LambdaMappingEntry {
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

impl ToProtobuf for LambdaStorageUpdate {
    type Protobuf = services::LambdaStorageUpdate;

    fn to_protobuf(&self) -> Self::Protobuf {
        match self {
            Self::StorageSlot(slot) => services::LambdaStorageUpdate {
                update: Some(services::lambda_storage_update::Update::StorageSlot(
                    slot.to_protobuf(),
                )),
            },
            Self::MappingEntries(entries) => services::LambdaStorageUpdate {
                update: Some(services::lambda_storage_update::Update::MappingEntries(
                    entries.to_protobuf(),
                )),
            },
        }
    }
}

impl FromProtobuf<services::LambdaStorageUpdate> for LambdaStorageUpdate {
    fn from_protobuf(pb: services::LambdaStorageUpdate) -> crate::Result<Self> {
        match pb.update {
            Some(services::lambda_storage_update::Update::StorageSlot(slot)) => {
                Ok(Self::StorageSlot(LambdaStorageSlot::from_protobuf(slot)?))
            }
            Some(services::lambda_storage_update::Update::MappingEntries(entries)) => {
                Ok(Self::MappingEntries(LambdaMappingEntries::from_protobuf(entries)?))
            }
            None => Err(crate::Error::basic_parse(
                "LambdaStorageUpdate must have either storage_slot or mapping_entries",
            )),
        }
    }
}

impl ToProtobuf for LambdaMappingEntries {
    type Protobuf = services::LambdaMappingEntries;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::LambdaMappingEntries {
            mapping_slot: self.mapping_slot.clone(),
            entries: self.entries.iter().map(|entry| entry.to_protobuf()).collect(),
        }
    }
}

impl FromProtobuf<services::LambdaMappingEntries> for LambdaMappingEntries {
    fn from_protobuf(pb: services::LambdaMappingEntries) -> crate::Result<Self> {
        let entries = pb
            .entries
            .into_iter()
            .map(LambdaMappingEntry::from_protobuf)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { mapping_slot: pb.mapping_slot, entries })
    }
}

impl ToProtobuf for LambdaMappingEntry {
    type Protobuf = services::LambdaMappingEntry;

    fn to_protobuf(&self) -> Self::Protobuf {
        let entry_key = if let Some(key) = &self.key {
            Some(services::lambda_mapping_entry::EntryKey::Key(key.clone()))
        } else if let Some(preimage) = &self.preimage {
            Some(services::lambda_mapping_entry::EntryKey::Preimage(preimage.clone()))
        } else {
            None
        };

        services::LambdaMappingEntry { entry_key, value: self.value.clone().unwrap_or_default() }
    }
}

impl FromProtobuf<services::LambdaMappingEntry> for LambdaMappingEntry {
    fn from_protobuf(pb: services::LambdaMappingEntry) -> crate::Result<Self> {
        let (key, preimage) = match pb.entry_key {
            Some(services::lambda_mapping_entry::EntryKey::Key(k)) => (Some(k), None),
            Some(services::lambda_mapping_entry::EntryKey::Preimage(p)) => (None, Some(p)),
            None => (None, None),
        };

        Ok(Self { key, value: if pb.value.is_empty() { None } else { Some(pb.value) }, preimage })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda_mapping_entry_creation() {
        let entry = LambdaMappingEntry::new(Some(vec![1, 2, 3]), Some(vec![4, 5, 6]));
        assert_eq!(entry.key, Some(vec![1, 2, 3]));
        assert_eq!(entry.value, Some(vec![4, 5, 6]));
        assert_eq!(entry.preimage, None);
    }

    #[test]
    fn test_lambda_mapping_entry_with_preimage() {
        let mut entry = LambdaMappingEntry::new(None, Some(vec![10, 11, 12]));
        entry.set_preimage(vec![7, 8, 9]);
        assert_eq!(entry.key, None);
        assert_eq!(entry.preimage, Some(vec![7, 8, 9]));
        assert_eq!(entry.value, Some(vec![10, 11, 12]));
    }

    #[test]
    fn test_lambda_mapping_entry_setters() {
        let mut entry = LambdaMappingEntry::new(None, None);
        entry.set_key(vec![7, 8, 9]).set_value(vec![10, 11, 12]);

        assert_eq!(entry.key, Some(vec![7, 8, 9]));
        assert_eq!(entry.value, Some(vec![10, 11, 12]));
        assert_eq!(entry.preimage, None);
    }

    #[test]
    fn test_lambda_mapping_entry_key_preimage_mutual_exclusion() {
        let mut entry = LambdaMappingEntry::new(Some(vec![1, 2, 3]), None);
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
    fn test_lambda_mapping_entry_protobuf_roundtrip() {
        let original = LambdaMappingEntry::new(Some(vec![1, 2, 3]), Some(vec![4, 5, 6]));
        let protobuf = original.to_protobuf();
        let reconstructed = LambdaMappingEntry::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
