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
    #[allow(unreachable_patterns)]
    fn from_protobuf(pb: services::EvmHookSpec) -> crate::Result<Self> {
        let contract_id = match pb.bytecode_source {
            Some(services::evm_hook_spec::BytecodeSource::ContractId(id)) => {
                Some(ContractId::from_protobuf(id)?)
            }
            // For future unsupported bytecode sources.
            Some(_) => {
                return Err(crate::Error::from_protobuf("unsupported EvmHookSpec.bytecode_source"));
            }
            None => None,
        };
        Ok(Self { contract_id })
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::ContractId;

    #[test]
    fn new_with_contract_id_sets_field() {
        let cid = ContractId::new(0, 0, 123);
        let spec = EvmHookSpec::new(Some(cid));
        assert_eq!(spec.contract_id, Some(cid));
    }

    #[test]
    fn new_without_contract_id_sets_none() {
        let spec = EvmHookSpec::new(None);
        assert!(spec.contract_id.is_none());
    }

    #[test]
    fn to_protobuf_with_contract_id_sets_bytecode_source() {
        let cid = ContractId::new(0, 0, 321);
        let spec = EvmHookSpec::new(Some(cid));
        let pb = spec.to_protobuf();

        let got = match pb.bytecode_source {
            Some(hedera_proto::services::evm_hook_spec::BytecodeSource::ContractId(id)) => {
                Some(ContractId::from_protobuf(id).unwrap())
            }
            None => None,
        };

        assert_eq!(got, Some(cid));
    }

    #[test]
    fn to_protobuf_without_contract_id_sets_none() {
        let spec = EvmHookSpec::new(None);
        let pb = spec.to_protobuf();
        assert!(pb.bytecode_source.is_none());
    }

    #[test]
    fn from_protobuf_with_contract_id_parses() {
        let cid = ContractId::new(0, 0, 555);
        let pb = hedera_proto::services::EvmHookSpec {
            bytecode_source: Some(
                hedera_proto::services::evm_hook_spec::BytecodeSource::ContractId(
                    cid.to_protobuf(),
                ),
            ),
        };

        let spec = EvmHookSpec::from_protobuf(pb).unwrap();
        assert_eq!(spec.contract_id, Some(cid));
    }

    #[test]
    fn from_protobuf_without_contract_id_parses_none() {
        let pb = hedera_proto::services::EvmHookSpec { bytecode_source: None };
        let spec = EvmHookSpec::from_protobuf(pb).unwrap();
        assert!(spec.contract_id.is_none());
    }

    #[test]
    fn protobuf_roundtrip() {
        let cid = ContractId::new(0, 0, 999);
        let original = EvmHookSpec::new(Some(cid));
        let pb = original.to_protobuf();
        let reconstructed = EvmHookSpec::from_protobuf(pb).unwrap();
        assert_eq!(original, reconstructed);
    }
}
