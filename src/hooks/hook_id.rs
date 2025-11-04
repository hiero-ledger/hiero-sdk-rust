use hedera_proto::services;

use crate::hooks::HookEntityId;
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// A hook identifier containing an entity ID and hook ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HookId {
    /// The entity ID associated with this hook.
    pub entity_id: Option<HookEntityId>,
    /// The hook ID number.
    pub hook_id: i64,
}

impl HookId {
    /// Create a new `HookId`.
    pub fn new(entity_id: Option<HookEntityId>, hook_id: i64) -> Self {
        Self { entity_id, hook_id }
    }
}

impl ToProtobuf for HookId {
    type Protobuf = services::HookId;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::HookId {
            entity_id: self.entity_id.as_ref().map(|id| id.to_protobuf()),
            hook_id: self.hook_id,
        }
    }
}

impl FromProtobuf<services::HookId> for HookId {
    fn from_protobuf(pb: services::HookId) -> crate::Result<Self> {
        let entity_id = pb.entity_id.map(HookEntityId::from_protobuf).transpose()?;

        Ok(Self { entity_id, hook_id: pb.hook_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountId;

    #[test]
    fn test_hook_id_creation() {
        let entity_id = HookEntityId::new(Some(AccountId::new(0, 0, 123)));
        let hook_id = HookId::new(Some(entity_id.clone()), 456);

        assert_eq!(hook_id.entity_id, Some(entity_id));
        assert_eq!(hook_id.hook_id, 456);
    }

    #[test]
    fn test_hook_id_with_entity_id_only() {
        let entity_id = HookEntityId::new(Some(AccountId::new(0, 0, 789)));
        let hook_id = HookId::new(Some(entity_id.clone()), 123);

        assert_eq!(hook_id.entity_id, Some(entity_id));
        assert_eq!(hook_id.hook_id, 123);
    }

    #[test]
    fn test_hook_id_with_hook_id_only() {
        let hook_id = HookId::new(None, 999);

        assert_eq!(hook_id.entity_id, None);
        assert_eq!(hook_id.hook_id, 999);
    }

    #[test]
    fn test_hook_id_protobuf_roundtrip() {
        let entity_id = HookEntityId::new(Some(AccountId::new(0, 0, 111)));
        let original = HookId::new(Some(entity_id), 222);
        let protobuf = original.to_protobuf();
        let reconstructed = HookId::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
