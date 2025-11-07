/// Types of NFT hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NftHookType {
    /// A single call made before attempting the NFT transfer, to a hook on the sender account.
    PreHookSender = 0,
    /// Two calls - first before attempting the NFT transfer (allowPre), and second after
    /// attempting the NFT transfer (allowPost) on the sender account.
    PrePostHookSender = 1,
    /// A single call made before attempting the NFT transfer, to a hook on the receiver account.
    PreHookReceiver = 2,
    /// Two calls - first before attempting the NFT transfer (allowPre), and second after
    /// attempting the NFT transfer (allowPost) on the receiver account.
    PrePostHookReceiver = 3,
}

impl NftHookType {
    /// Returns the numeric value of the hook type.
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_hook_type_values() {
        assert_eq!(NftHookType::PreHookSender.value(), 0);
        assert_eq!(NftHookType::PrePostHookSender.value(), 1);
        assert_eq!(NftHookType::PreHookReceiver.value(), 2);
        assert_eq!(NftHookType::PrePostHookReceiver.value(), 3);
    }
}
