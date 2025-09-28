use crate::account::AccountId;
use crate::protobuf::services;
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// A hook entity identifier that can contain an account ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HookEntityId {
    pub account_id: Option<AccountId>,
}

impl HookEntityId {
    pub fn new(account_id: Option<AccountId>) -> Self {
        Self { account_id }
    }
}

impl ToProtobuf for HookEntityId {
    type Protobuf = services::HookEntityId;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::HookEntityId { account_id: self.account_id.map(|id| id.to_protobuf()) }
    }
}

impl FromProtobuf<services::HookEntityId> for HookEntityId {
    fn from_protobuf(pb: services::HookEntityId) -> crate::Result<Self> {
        let account_id = pb.account_id.map(AccountId::from_protobuf).transpose()?;

        Ok(Self { account_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_entity_id_with_account_id() {
        let account_id = AccountId::new(0, 0, 123);
        let hook_entity_id = HookEntityId::with_account_id(account_id);

        assert_eq!(hook_entity_id.account_id, Some(account_id));
    }

    #[test]
    fn test_hook_entity_id_empty() {
        let hook_entity_id = HookEntityId::empty();

        assert_eq!(hook_entity_id.account_id, None);
    }

    #[test]
    fn test_hook_entity_id_protobuf_roundtrip() {
        let account_id = AccountId::new(0, 0, 456);
        let original = HookEntityId::with_account_id(account_id);

        let protobuf = original.to_protobuf();
        let reconstructed = HookEntityId::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
