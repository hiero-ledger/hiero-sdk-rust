/// Hook extension points that can be used to register hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum HookExtensionPoint {
    /// Account allowance hook extension point.
    AccountAllowanceHook = 0,
}

impl HookExtensionPoint {
    /// Get the numeric value of the extension point.
    pub const fn value(self) -> u32 {
        self as u32
    }

    /// Create a `HookExtensionPoint` from a numeric value.
    ///
    /// # Errors
    /// Returns `None` if the value is not a valid extension point.
    pub const fn from_value(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::AccountAllowanceHook),
            _ => None,
        }
    }
}

impl From<HookExtensionPoint> for u32 {
    fn from(point: HookExtensionPoint) -> Self {
        point.value()
    }
}

impl TryFrom<u32> for HookExtensionPoint {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_value(value).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_extension_point_values() {
        assert_eq!(HookExtensionPoint::AccountAllowanceHook.value(), 0);
    }

    #[test]
    fn test_hook_extension_point_from_value() {
        assert_eq!(
            HookExtensionPoint::from_value(0),
            Some(HookExtensionPoint::AccountAllowanceHook)
        );
        assert_eq!(HookExtensionPoint::from_value(1), None);
    }

    #[test]
    fn test_hook_extension_point_conversions() {
        let point = HookExtensionPoint::AccountAllowanceHook;
        let value: u32 = point.into();
        assert_eq!(value, 0);

        let point_from_value = HookExtensionPoint::try_from(0).unwrap();
        assert_eq!(point_from_value, HookExtensionPoint::AccountAllowanceHook);

        assert!(HookExtensionPoint::try_from(999).is_err());
    }
}
