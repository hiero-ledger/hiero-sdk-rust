use crate::hooks::HookEntityId;
use crate::protobuf::services;
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
    pub hook_id: Option<i64>,
}

impl HookId {
    /// Create a new `HookId`.
    pub fn new(entity_id: Option<HookEntityId>, hook_id: Option<i64>) -> Self {
        Self { entity_id, hook_id }
    }

    /// Create a new `HookId` with an entity ID.
    pub fn with_entity_id(entity_id: HookEntityId) -> Self {
        Self { entity_id: Some(entity_id), hook_id: None }
    }

    /// Create a new `HookId` with a hook ID.
    pub fn with_hook_id(hook_id: i64) -> Self {
        Self { entity_id: None, hook_id: Some(hook_id) }
    }

    /// Create a new `HookId` with both entity ID and hook ID.
    pub fn with_both(entity_id: HookEntityId, hook_id: i64) -> Self {
        Self { entity_id: Some(entity_id), hook_id: Some(hook_id) }
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
        let entity_id = HookEntityId::with_account_id(AccountId::new(0, 0, 123));
        let hook_id = HookId::with_both(entity_id.clone(), 456);

        assert_eq!(hook_id.entity_id, Some(entity_id));
        assert_eq!(hook_id.hook_id, Some(456));
    }

    #[test]
    fn test_hook_id_with_entity_id_only() {
        let entity_id = HookEntityId::with_account_id(AccountId::new(0, 0, 789));
        let hook_id = HookId::with_entity_id(entity_id.clone());

        assert_eq!(hook_id.entity_id, Some(entity_id));
        assert_eq!(hook_id.hook_id, None);
    }

    #[test]
    fn test_hook_id_with_hook_id_only() {
        let hook_id = HookId::with_hook_id(999);

        assert_eq!(hook_id.entity_id, None);
        assert_eq!(hook_id.hook_id, Some(999));
    }

    #[test]
    fn test_hook_id_protobuf_roundtrip() {
        let entity_id = HookEntityId::with_account_id(AccountId::new(0, 0, 111));
        let original = HookId::with_both(entity_id, 222);
        let protobuf = original.to_protobuf();
        let reconstructed = HookId::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
