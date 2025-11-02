#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum HookType {
    PreHook = 0,
    PrePostHook = 1,
    PreHookSender = 2,
    PrePostHookSender = 3,
    PreHookReceiver = 4,
    PrePostHookReceiver = 5,
}

impl HookType {
    pub const fn value(self) -> u32 {
        self as u32
    }

    pub const fn from_value(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::PreHook),
            1 => Some(Self::PrePostHook),
            2 => Some(Self::PreHookSender),
            3 => Some(Self::PrePostHookSender),
            4 => Some(Self::PreHookReceiver),
            5 => Some(Self::PrePostHookReceiver),
            _ => None,
        }
    }
}

impl From<HookType> for u32 {
    fn from(hook_type: HookType) -> Self {
        hook_type.value()
    }
}

impl TryFrom<u32> for HookType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_value(value).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_values() {
        assert_eq!(HookType::PreHook.value(), 0);
        assert_eq!(HookType::PrePostHook.value(), 1);
        assert_eq!(HookType::PreHookSender.value(), 2);
        assert_eq!(HookType::PrePostHookSender.value(), 3);
        assert_eq!(HookType::PreHookReceiver.value(), 4);
        assert_eq!(HookType::PrePostHookReceiver.value(), 5);
    }

    #[test]
    fn test_hook_type_from_value() {
        assert_eq!(HookType::from_value(0), Some(HookType::PreHook));
        assert_eq!(HookType::from_value(1), Some(HookType::PrePostHook));
        assert_eq!(HookType::from_value(2), Some(HookType::PreHookSender));
        assert_eq!(HookType::from_value(3), Some(HookType::PrePostHookSender));
        assert_eq!(HookType::from_value(4), Some(HookType::PreHookReceiver));
        assert_eq!(HookType::from_value(5), Some(HookType::PrePostHookReceiver));
        assert_eq!(HookType::from_value(6), None);
    }

    #[test]
    fn test_hook_type_conversions() {
        let hook_type = HookType::PrePostHook;
        let value: u32 = hook_type.into();
        assert_eq!(value, 1);

        let hook_type_from_value = HookType::try_from(1).unwrap();
        assert_eq!(hook_type_from_value, HookType::PrePostHook);

        assert!(HookType::try_from(999).is_err());
    }
}
