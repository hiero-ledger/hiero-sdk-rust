use hedera_proto::services;

use crate::hooks::{
    FungibleHookType,
    HookCall,
};
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// A typed hook call for fungible (HBAR and FT) transfers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FungibleHookCall {
    /// The underlying hook call data.
    pub hook_call: HookCall,
    /// The type of fungible hook.
    pub hook_type: FungibleHookType,
}

impl FungibleHookCall {
    /// Create a new `FungibleHookCall`.
    pub fn new(hook_call: HookCall, hook_type: FungibleHookType) -> Self {
        Self { hook_call, hook_type }
    }

    /// Internal method to create from protobuf with a known type.
    pub(crate) fn from_protobuf_with_type(
        pb: services::HookCall,
        hook_type: FungibleHookType,
    ) -> crate::Result<Self> {
        Ok(Self { hook_call: HookCall::from_protobuf(pb)?, hook_type })
    }
}

impl ToProtobuf for FungibleHookCall {
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
    fn test_fungible_hook_call_creation() {
        let hook_id = 123;
        let hook_type = FungibleHookType::PreTxAllowanceHook;
        let mut hook_call_obj = HookCall::new(None, None);
        hook_call_obj.set_hook_id(hook_id);
        let hook_call = FungibleHookCall::new(hook_call_obj, hook_type);

        assert_eq!(hook_call.hook_call.hook_id, Some(hook_id));
        assert_eq!(hook_call.hook_type, hook_type);
    }

    #[test]
    fn test_fungible_hook_call_with_call() {
        let call_data = vec![1, 2, 3, 4, 5];
        let evm_call = EvmHookCall::new(Some(call_data));
        let hook_type = FungibleHookType::PrePostTxAllowanceHook;
        let mut hook_call_obj = HookCall::new(None, None);
        hook_call_obj.set_call(evm_call.clone());
        let hook_call = FungibleHookCall::new(hook_call_obj, hook_type);

        assert_eq!(hook_call.hook_call.call, Some(evm_call));
        assert_eq!(hook_call.hook_type, hook_type);
    }
}
