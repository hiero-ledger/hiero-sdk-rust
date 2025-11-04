use hedera_proto::services;

use crate::hooks::{
    HookExtensionPoint,
    LambdaEvmHook,
};
use crate::key::Key;
use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// Details for creating a hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookCreationDetails {
    /// The extension point for the hook.
    pub extension_point: HookExtensionPoint,
    /// The ID to create the hook at.
    pub hook_id: i64,
    /// The hook implementation (currently only Lambda EVM hooks).
    pub lambda_evm_hook: Option<LambdaEvmHook>,
    /// Admin key for the hook (if any).
    pub admin_key: Option<Key>,
}

impl HookCreationDetails {
    /// Create a new `HookCreationDetails`.
    pub fn new(
        extension_point: HookExtensionPoint,
        hook_id: i64,
        lambda_evm_hook: Option<LambdaEvmHook>,
    ) -> Self {
        Self { extension_point, hook_id, lambda_evm_hook, admin_key: None }
    }
}

impl ToProtobuf for HookCreationDetails {
    type Protobuf = services::HookCreationDetails;

    fn to_protobuf(&self) -> Self::Protobuf {
        let hook = self
            .lambda_evm_hook
            .as_ref()
            .map(|h| services::hook_creation_details::Hook::LambdaEvmHook(h.to_protobuf()));

        services::HookCreationDetails {
            extension_point: self.extension_point as i32,
            hook_id: self.hook_id,
            hook,
            admin_key: self.admin_key.as_ref().map(|k| k.to_protobuf()),
        }
    }
}

impl FromProtobuf<services::HookCreationDetails> for HookCreationDetails {
    fn from_protobuf(pb: services::HookCreationDetails) -> crate::Result<Self> {
        let extension_point = HookExtensionPoint::try_from(pb.extension_point)?;

        let lambda_evm_hook = match pb.hook {
            Some(services::hook_creation_details::Hook::LambdaEvmHook(hook)) => {
                Some(LambdaEvmHook::from_protobuf(hook)?)
            }
            None => None,
        };

        let admin_key = pb.admin_key.map(Key::from_protobuf).transpose()?;

        Ok(Self { extension_point, hook_id: pb.hook_id, lambda_evm_hook, admin_key })
    }
}
