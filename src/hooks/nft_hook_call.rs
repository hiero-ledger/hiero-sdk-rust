use hedera_proto::services;

use crate::hooks::{
    HookCall,
    NftHookType,
};
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// A typed hook call for NFT transfers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftHookCall {
    /// The underlying hook call data.
    pub hook_call: HookCall,
    /// The type of NFT hook.
    pub hook_type: NftHookType,
}

impl NftHookCall {
    /// Create a new `NftHookCall`.
    pub fn new(hook_call: HookCall, hook_type: NftHookType) -> Self {
        Self { hook_call, hook_type }
    }

    /// Internal method to create from protobuf with a known type.
    pub(crate) fn from_protobuf_with_type(
        pb: services::HookCall,
        hook_type: NftHookType,
    ) -> crate::Result<Self> {
        Ok(Self { hook_call: HookCall::from_protobuf(pb)?, hook_type })
    }
}

impl ToProtobuf for NftHookCall {
    type Protobuf = services::HookCall;

    fn to_protobuf(&self) -> Self::Protobuf {
        self.hook_call.to_protobuf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::EvmHookCall;

    #[test]
    fn test_nft_hook_call_creation() {
        let hook_id = 123;
        let hook_type = NftHookType::PreHookSender;
        let mut hook_call_obj = HookCall::new(None, None);
        hook_call_obj.set_hook_id(hook_id);
        let hook_call = NftHookCall::new(hook_call_obj, hook_type);

        assert_eq!(hook_call.hook_call.hook_id, Some(hook_id));
        assert_eq!(hook_call.hook_type, hook_type);
    }

    #[test]
    fn test_nft_hook_call_with_call() {
        let call_data = vec![1, 2, 3, 4, 5];
        let mut evm_call = EvmHookCall::new(Some(call_data));
        evm_call.set_gas_limit(0);
        let hook_type = NftHookType::PrePostHookReceiver;
        let mut hook_call_obj = HookCall::new(None, None);
        hook_call_obj.set_call(evm_call.clone());
        let hook_call = NftHookCall::new(hook_call_obj, hook_type);

        assert_eq!(hook_call.hook_call.call, Some(evm_call));
        assert_eq!(hook_call.hook_type, hook_type);
    }
}
