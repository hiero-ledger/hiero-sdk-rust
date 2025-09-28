use crate::hooks::EvmHookCall;
use crate::protobuf::services;
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// A hook call containing a hook ID and call data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookCall {
    /// The ID of the hook to call.
    pub hook_id: Option<i64>,
    /// The call data for the hook.
    pub call: Option<EvmHookCall>,
}

impl HookCall {
    /// Create a new `HookCall`.
    pub fn new(hook_id: Option<i64>, call: Option<EvmHookCall>) -> Self {
        Self { hook_id, call }
    }

    /// Create a new `HookCall` with a hook ID.
    pub fn with_hook_id(hook_id: i64) -> Self {
        Self { hook_id: Some(hook_id), call: None }
    }

    /// Create a new `HookCall` with call data.
    pub fn with_call(call: EvmHookCall) -> Self {
        Self { hook_id: None, call: Some(call) }
    }

    /// Set the hook ID.
    pub fn set_hook_id(&mut self, hook_id: i64) -> &mut Self {
        self.hook_id = Some(hook_id);
        self
    }

    /// Set the call data.
    pub fn set_call(&mut self, call: EvmHookCall) -> &mut Self {
        self.call = Some(call);
        self
    }
}

impl ToProtobuf for HookCall {
    type Protobuf = services::HookCall;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::HookCall {
            hook_id: self.hook_id,
            evm_hook_call: self.call.as_ref().map(|call| call.to_protobuf()),
        }
    }
}

impl FromProtobuf<services::HookCall> for HookCall {
    fn from_protobuf(pb: services::HookCall) -> crate::Result<Self> {
        let call = pb.evm_hook_call.map(EvmHookCall::from_protobuf).transpose()?;

        Ok(Self { hook_id: pb.hook_id, call })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_call_creation() {
        let hook_id = 123;
        let call_data = vec![1, 2, 3, 4, 5];
        let evm_call = EvmHookCall::with_call_data(call_data);

        let hook_call = HookCall::new(Some(hook_id), Some(evm_call.clone()));

        assert_eq!(hook_call.hook_id, Some(hook_id));
        assert_eq!(hook_call.call, Some(evm_call));
    }

    #[test]
    fn test_hook_call_with_hook_id() {
        let hook_id = 456;
        let hook_call = HookCall::with_hook_id(hook_id);

        assert_eq!(hook_call.hook_id, Some(hook_id));
        assert_eq!(hook_call.call, None);
    }

    #[test]
    fn test_hook_call_with_call() {
        let call_data = vec![6, 7, 8, 9, 10];
        let evm_call = EvmHookCall::with_call_data(call_data.clone());
        let hook_call = HookCall::with_call(evm_call.clone());

        assert_eq!(hook_call.hook_id, None);
        assert_eq!(hook_call.call, Some(evm_call));
    }

    #[test]
    fn test_hook_call_setters() {
        let mut hook_call = HookCall::new(None, None);
        let hook_id = 789;
        let call_data = vec![11, 12, 13, 14, 15];
        let evm_call = EvmHookCall::with_call_data(call_data);

        hook_call.set_hook_id(hook_id).set_call(evm_call.clone());

        assert_eq!(hook_call.hook_id, Some(hook_id));
        assert_eq!(hook_call.call, Some(evm_call));
    }

    #[test]
    fn test_hook_call_protobuf_roundtrip() {
        let hook_id = 999;
        let call_data = vec![16, 17, 18, 19, 20];
        let evm_call = EvmHookCall::with_call_data(call_data);
        let original = HookCall::new(Some(hook_id), Some(evm_call));

        let protobuf = original.to_protobuf();
        let reconstructed = HookCall::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
