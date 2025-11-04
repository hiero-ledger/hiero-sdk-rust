use hedera_proto::services;

use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// A slot in the storage of a lambda EVM hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaStorageSlot {
    /// The key of the slot.
    pub key: Vec<u8>,

    /// The value of the slot.
    pub value: Vec<u8>,
}

impl LambdaStorageSlot {
    /// Create a new `LambdaStorageSlot`.
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { key, value }
    }

    /// Get the key.
    pub fn get_key(&self) -> &[u8] {
        &self.key
    }

    /// Get the value.
    pub fn get_value(&self) -> &[u8] {
        &self.value
    }

    /// Set the value.
    pub fn set_value(&mut self, value: Vec<u8>) {
        self.value = value;
    }

    /// Set the key.
    pub fn set_key(&mut self, key: Vec<u8>) {
        self.key = key;
    }
}

impl ToProtobuf for LambdaStorageSlot {
    type Protobuf = services::LambdaStorageSlot;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::LambdaStorageSlot { key: self.key.clone(), value: self.value.clone() }
    }
}

impl FromProtobuf<services::LambdaStorageSlot> for LambdaStorageSlot {
    fn from_protobuf(pb: services::LambdaStorageSlot) -> crate::Result<Self> {
        Ok(Self { key: pb.key, value: pb.value })
    }
}
