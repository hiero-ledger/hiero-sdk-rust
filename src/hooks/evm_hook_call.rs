use hedera_proto::services;

use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// An EVM hook call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmHookCall {
    /// The call data for the EVM hook.
    pub call_data: Option<Vec<u8>>,
    /// The gas limit for the hook call.
    pub gas_limit: Option<u64>,
}

impl EvmHookCall {
    /// Create a new `EvmHookCall`.
    pub fn new(call_data: Option<Vec<u8>>) -> Self {
        Self { call_data, gas_limit: None }
    }

    /// Set the call data for the hook.
    pub fn set_call_data(&mut self, call_data: Vec<u8>) -> &mut Self {
        self.call_data = Some(call_data);
        self
    }

    /// Set the gas limit for the hook call.
    pub fn set_gas_limit(&mut self, gas_limit: u64) -> &mut Self {
        self.gas_limit = Some(gas_limit);
        self
    }
}

impl ToProtobuf for EvmHookCall {
    type Protobuf = services::EvmHookCall;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::EvmHookCall {
            data: self.call_data.clone().unwrap_or_default(),
            gas_limit: self.gas_limit.unwrap_or(0),
        }
    }
}

impl FromProtobuf<services::EvmHookCall> for EvmHookCall {
    fn from_protobuf(pb: services::EvmHookCall) -> crate::Result<Self> {
        Ok(Self {
            call_data: if pb.data.is_empty() { None } else { Some(pb.data) },
            gas_limit: Some(pb.gas_limit),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_hook_call_creation() {
        let call_data = vec![1, 2, 3, 4, 5];
        let hook_call = EvmHookCall::new(Some(call_data.clone()));

        assert_eq!(hook_call.call_data, Some(call_data));
    }

    #[test]
    fn test_evm_hook_call_setters() {
        let mut hook_call = EvmHookCall::new(None);
        let call_data = vec![6, 7, 8, 9, 10];

        hook_call.set_call_data(call_data.clone());
        assert_eq!(hook_call.call_data, Some(call_data));
    }

    #[test]
    fn test_evm_hook_call_protobuf_roundtrip() {
        let call_data = vec![11, 12, 13, 14, 15];
        let original = EvmHookCall::new(Some(call_data));

        let protobuf = original.to_protobuf();
        let reconstructed = EvmHookCall::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
