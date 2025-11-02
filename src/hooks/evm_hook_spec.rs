use hedera_proto::services;

use crate::contract::ContractId;
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// Shared specifications for an EVM hook. May be used for any extension point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmHookSpec {
    /// The id of a contract that implements the extension point API with EVM bytecode.
    pub contract_id: Option<ContractId>,
}

impl EvmHookSpec {
    /// Create a new `EvmHookSpec`.
    pub fn new(contract_id: Option<ContractId>) -> Self {
        Self { contract_id }
    }
}

impl ToProtobuf for EvmHookSpec {
    type Protobuf = services::EvmHookSpec;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::EvmHookSpec {
            bytecode_source: self
                .contract_id
                .as_ref()
                .map(|id| services::evm_hook_spec::BytecodeSource::ContractId(id.to_protobuf())),
        }
    }
}

impl FromProtobuf<services::EvmHookSpec> for EvmHookSpec {
    fn from_protobuf(pb: services::EvmHookSpec) -> crate::Result<Self> {
        let contract_id = match pb.bytecode_source {
            Some(services::evm_hook_spec::BytecodeSource::ContractId(id)) => {
                Some(ContractId::from_protobuf(id)?)
            }
            None => None,
        };

        Ok(Self { contract_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::ContractId;

    #[test]
    fn test_evm_hook_spec_creation() {
        let contract_id = ContractId::new(0, 0, 123);
        let spec = EvmHookSpec::new(Some(contract_id.clone()));

        assert_eq!(spec.contract_id, Some(contract_id));
    }

    #[test]
    fn test_evm_hook_spec_protobuf_roundtrip() {
        let contract_id = ContractId::new(0, 0, 456);
        let original = EvmHookSpec::new(Some(contract_id));
        let protobuf = original.to_protobuf();
        let reconstructed = EvmHookSpec::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
