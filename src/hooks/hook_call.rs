use hedera_proto::services;

use crate::hooks::{
    EvmHookCall,
    HookId,
};
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// A hook call containing a hook ID and call data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookCall {
    /// The full hook ID (entity_id + numeric id).
    pub full_hook_id: Option<HookId>,
    /// The numeric ID of the hook (when entity is implied).
    pub hook_id: Option<i64>,
    /// The call data for the hook.
    pub call: Option<EvmHookCall>,
}

impl HookCall {
    /// Create a new `HookCall`.
    pub fn new(hook_id: Option<i64>, call: Option<EvmHookCall>) -> Self {
        Self { full_hook_id: None, hook_id, call }
    }

    /// Set the full hook ID (clears hook_id if set).
    pub fn set_full_hook_id(&mut self, full_hook_id: HookId) -> &mut Self {
        self.full_hook_id = Some(full_hook_id);
        self.hook_id = None; // Clear hook_id since they're mutually exclusive
        self
    }

    /// Set the hook ID (clears full_hook_id if set).
    pub fn set_hook_id(&mut self, hook_id: i64) -> &mut Self {
        self.hook_id = Some(hook_id);
        self.full_hook_id = None; // Clear full_hook_id since they're mutually exclusive
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
        let id = if let Some(full_hook_id) = &self.full_hook_id {
            Some(services::hook_call::Id::FullHookId(full_hook_id.to_protobuf()))
        } else if let Some(hook_id) = self.hook_id {
            Some(services::hook_call::Id::HookId(hook_id))
        } else {
            None
        };

        let call_spec = self
            .call
            .as_ref()
            .map(|call| services::hook_call::CallSpec::EvmHookCall(call.to_protobuf()));

        services::HookCall { id, call_spec }
    }
}

impl FromProtobuf<services::HookCall> for HookCall {
    fn from_protobuf(pb: services::HookCall) -> crate::Result<Self> {
        let (full_hook_id, hook_id) = match pb.id {
            Some(services::hook_call::Id::FullHookId(id)) => {
                (Some(HookId::from_protobuf(id)?), None)
            }
            Some(services::hook_call::Id::HookId(id)) => (None, Some(id)),
            None => (None, None),
        };

        let call = match pb.call_spec {
            Some(services::hook_call::CallSpec::EvmHookCall(call)) => {
                Some(EvmHookCall::from_protobuf(call)?)
            }
            None => None,
        };

        Ok(Self { full_hook_id, hook_id, call })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_call_creation() {
        let hook_id = 123;
        let call_data = vec![1, 2, 3, 4, 5];
        let evm_call = EvmHookCall::new(Some(call_data));

        let hook_call = HookCall::new(Some(hook_id), Some(evm_call.clone()));

        assert_eq!(hook_call.hook_id, Some(hook_id));
        assert_eq!(hook_call.call, Some(evm_call));
    }

    #[test]
    fn test_hook_call_with_hook_id() {
        let hook_id = 456;
        let mut hook_call = HookCall::new(None, None);
        hook_call.set_hook_id(hook_id);

        assert_eq!(hook_call.hook_id, Some(hook_id));
        assert_eq!(hook_call.call, None);
    }

    #[test]
    fn test_hook_call_with_call() {
        let call_data = vec![6, 7, 8, 9, 10];
        let evm_call = EvmHookCall::new(Some(call_data.clone()));
        let mut hook_call = HookCall::new(None, None);
        hook_call.set_call(evm_call.clone());

        assert_eq!(hook_call.hook_id, None);
        assert_eq!(hook_call.call, Some(evm_call));
    }

    #[test]
    fn test_hook_call_setters() {
        let mut hook_call = HookCall::new(None, None);
        let hook_id = 789;
        let call_data = vec![11, 12, 13, 14, 15];
        let evm_call = EvmHookCall::new(Some(call_data));

        hook_call.set_hook_id(hook_id).set_call(evm_call.clone());

        assert_eq!(hook_call.hook_id, Some(hook_id));
        assert_eq!(hook_call.call, Some(evm_call));
    }

    #[test]
    fn test_hook_call_protobuf_roundtrip() {
        let hook_id = 999;
        let call_data = vec![16, 17, 18, 19, 20];
        let evm_call = EvmHookCall::new(Some(call_data));
        let original = HookCall::new(Some(hook_id), Some(evm_call));

        let protobuf = original.to_protobuf();
        let reconstructed = HookCall::from_protobuf(protobuf).unwrap();

        assert_eq!(original, reconstructed);
    }
}
