/// Types of fungible (HBAR and FT) hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FungibleHookType {
    /// A single call made before attempting the transfer.
    PreTxAllowanceHook = 0,
    /// Two calls - first before attempting the transfer (allowPre), and second after
    /// attempting the transfer (allowPost).
    PrePostTxAllowanceHook = 1,
}

impl FungibleHookType {
    /// Returns the numeric value of the hook type.
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fungible_hook_type_values() {
        assert_eq!(FungibleHookType::PreTxAllowanceHook.value(), 0);
        assert_eq!(FungibleHookType::PrePostTxAllowanceHook.value(), 1);
    }
}
