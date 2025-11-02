use crate::account::AccountId;
use crate::contract::ContractId;
use crate::ledger_id::RefLedgerId;
use hedera_proto::services;
use crate::{
    Error,
    FromProtobuf,
    ToProtobuf,
    ValidateChecksums,
};

/// A hook entity identifier that can contain an account ID or contract ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HookEntityId {
    pub account_id: Option<AccountId>,
    pub contract_id: Option<ContractId>,
}

impl HookEntityId {
    pub fn new(account_id: Option<AccountId>) -> Self {
        Self { account_id, contract_id: None }
    }

    pub fn empty() -> Self {
        Self { account_id: None, contract_id: None }
    }
}

impl ToProtobuf for HookEntityId {
    type Protobuf = services::HookEntityId;

    fn to_protobuf(&self) -> Self::Protobuf {
        let entity_id = if let Some(account_id) = &self.account_id {
            Some(services::hook_entity_id::EntityId::AccountId(account_id.to_protobuf()))
        } else if let Some(contract_id) = &self.contract_id {
            Some(services::hook_entity_id::EntityId::ContractId(contract_id.to_protobuf()))
        } else {
            None
        };

        services::HookEntityId { entity_id }
    }
}

impl FromProtobuf<services::HookEntityId> for HookEntityId {
    fn from_protobuf(pb: services::HookEntityId) -> crate::Result<Self> {
        let (account_id, contract_id) = match pb.entity_id {
            Some(services::hook_entity_id::EntityId::AccountId(id)) => (Some(AccountId::from_protobuf(id)?), None),
            Some(services::hook_entity_id::EntityId::ContractId(id)) => (None, Some(ContractId::from_protobuf(id)?)),
            None => (None, None),
        };

        Ok(Self { account_id, contract_id })
    }
}

impl ValidateChecksums for HookEntityId {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        if let Some(account_id) = &self.account_id {
            account_id.validate_checksums(ledger_id)?;
        }
        if let Some(contract_id) = &self.contract_id {
            contract_id.validate_checksums(ledger_id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_entity_id_with_account_id() {
        let account_id = AccountId::new(0, 0, 123);
        let hook_entity_id = HookEntityId::new(Some(account_id));

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
        let original = HookEntityId::new(Some(account_id));

        let protobuf = original.to_protobuf();
        let reconstructed = HookEntityId::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
