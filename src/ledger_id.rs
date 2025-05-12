use std::borrow::Borrow;
use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;

use crate::Error;

enum KnownKind {
    Mainnet,
    Testnet,
    Previewnet,
}

/// DST that's semantically a [`LedgerId`].
///
/// `&Self` saves a pointer indirection vs `&LedgerId` because `LedgerId` is a `Box<[u8]>`, so `&LedgerId` is a `&Box<[u8]>`.
///
/// It also allows for const constructable `LedgerId`s.
///
/// Internal API, don't publically expose.
#[repr(transparent)]
#[derive(Eq, PartialEq)]
pub struct RefLedgerId([u8]);

impl RefLedgerId {
    pub(crate) const MAINNET: &'static Self = Self::new(&[0]);
    pub(crate) const TESTNET: &'static Self = Self::new(&[1]);
    pub(crate) const PREVIEWNET: &'static Self = Self::new(&[2]);

    pub const fn new(data: &[u8]) -> &Self {
        // safety: blessed by the standard library: see `path.rs`
        // https://github.com/rust-lang/rust/blob/3020239de947ec52677e9b4e853a6a9fc073d1f9/library/std/src/path.rs#L2037-L2039
        unsafe { &*(data as *const [u8] as *const RefLedgerId) }
    }

    pub fn new_boxed(data: Box<[u8]>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(data) as *mut RefLedgerId) }
    }

    const fn kind(&self) -> Option<KnownKind> {
        const MAINNET_BYTES: &[u8] = RefLedgerId::MAINNET.as_bytes();
        const TESTNET_BYTES: &[u8] = RefLedgerId::TESTNET.as_bytes();
        const PREVIEWNET_BYTES: &[u8] = RefLedgerId::PREVIEWNET.as_bytes();
        match self.as_bytes() {
            MAINNET_BYTES => Some(KnownKind::Mainnet),
            TESTNET_BYTES => Some(KnownKind::Testnet),
            PREVIEWNET_BYTES => Some(KnownKind::Previewnet),
            _ => None,
        }
    }

    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> From<&'a [u8]> for &'a RefLedgerId {
    fn from(value: &'a [u8]) -> Self {
        RefLedgerId::new(value)
    }
}

impl ToOwned for RefLedgerId {
    type Owned = LedgerId;

    fn to_owned(&self) -> Self::Owned {
        Self::Owned::from_bytes(self.0.to_vec())
    }
}

/// The ID of a Hiero Ledger.
#[derive(Eq, PartialEq)]
pub struct LedgerId(Box<RefLedgerId>);

impl LedgerId {
    #[must_use]
    const fn kind(&self) -> Option<KnownKind> {
        self.0.kind()
    }

    /// ID for the `mainnet` ledger.
    #[must_use]
    pub fn mainnet() -> Self {
        RefLedgerId::MAINNET.to_owned()
    }

    /// ID for the `testnet` ledger.
    #[must_use]
    pub fn testnet() -> Self {
        RefLedgerId::TESTNET.to_owned()
    }

    /// ID for the `previewnet` ledger.
    #[must_use]
    pub fn previewnet() -> Self {
        RefLedgerId::PREVIEWNET.to_owned()
    }

    /// Create a ledger ID from the given bytes.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(RefLedgerId::new_boxed(bytes.into_boxed_slice()))
    }

    /// Returns `true` if `self` is `mainnet`.
    #[must_use]
    pub fn is_mainnet(&self) -> bool {
        matches!(self.kind(), Some(KnownKind::Mainnet))
    }

    /// Returns `true` if `self` is `testnet`.
    #[must_use]
    pub fn is_testnet(&self) -> bool {
        matches!(self.kind(), Some(KnownKind::Testnet))
    }

    /// Returns `true` if `self` is `previewnet`.
    #[must_use]
    pub fn is_previewnet(&self) -> bool {
        matches!(self.kind(), Some(KnownKind::Previewnet))
    }

    /// Returns `true` if `self` is `mainnet`, `testnet`, or `previewnet`.
    #[must_use]
    pub fn is_known_network(&self) -> bool {
        self.kind().is_some()
    }

    #[must_use]
    pub(crate) fn as_ref_ledger_id(&self) -> &RefLedgerId {
        &self.0
    }

    // todo: remove so that we can have `LedgerId` be an enum internally?
    // that would make `mainnet`, `testnet`, and `previewnet`, all const constructable.
    // then we could use a
    #[must_use]
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Provides a byte representation of `self`.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl Clone for LedgerId {
    fn clone(&self) -> Self {
        Self::from_bytes(self.0.as_bytes().to_vec())
    }
}

impl AsRef<RefLedgerId> for LedgerId {
    fn as_ref(&self) -> &RefLedgerId {
        &self.0
    }
}

impl Borrow<RefLedgerId> for LedgerId {
    fn borrow(&self) -> &RefLedgerId {
        self.as_ref()
    }
}

impl Debug for LedgerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for LedgerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind() {
            Some(it) => f.write_str(match it {
                KnownKind::Mainnet => "mainnet",
                KnownKind::Testnet => "testnet",
                KnownKind::Previewnet => "previewnet",
            }),

            None => f.write_str(&hex::encode(self.as_bytes())),
        }
    }
}

impl FromStr for LedgerId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" => Ok(Self::mainnet()),
            "testnet" => Ok(Self::testnet()),
            "previewnet" => Ok(Self::previewnet()),
            _ => hex::decode(s).map(Self::from_bytes).map_err(Error::basic_parse),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::LedgerId;

    #[test]
    fn it_can_to_string() {
        assert_eq!(LedgerId::mainnet().to_string(), "mainnet");
        assert_eq!(LedgerId::testnet().to_string(), "testnet");
        assert_eq!(LedgerId::previewnet().to_string(), "previewnet");
        assert_eq!(
            LedgerId::from_bytes(vec![0x00, 0xFF, 0x00, 0xFF]).to_string().to_uppercase(),
            "00FF00FF"
        );
    }

    #[test]
    fn it_can_from_string() {
        assert_eq!(LedgerId::from_str("mainnet").unwrap(), LedgerId::mainnet());
        assert_eq!(LedgerId::from_str("testnet").unwrap(), LedgerId::testnet());
        assert_eq!(LedgerId::from_str("previewnet").unwrap(), LedgerId::previewnet());
        assert_eq!(
            LedgerId::from_str("00ff00ff").unwrap(),
            LedgerId::from_bytes(vec![0x00, 0xFF, 0x00, 0xFF])
        );
        assert_eq!(
            LedgerId::from_str("00FF00FF").unwrap(),
            LedgerId::from_bytes(vec![0x00, 0xFF, 0x00, 0xFF])
        );
    }

    #[test]
    fn it_can_to_bytes() {
        let bytes = vec![0x00, 0xFF, 0x00, 0xFF];
        assert_eq!(LedgerId::from_bytes(bytes.clone()).as_bytes(), &bytes);
    }
}
