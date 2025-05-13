// SPDX-License-Identifier: Apache-2.0

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};

use sha2::{
    Digest,
    Sha384,
};

/// The client-generated SHA-384 hash of a transaction that was submitted.
///
/// This can be used to lookup the transaction in an explorer.
#[derive(Copy, Clone, Hash)]
pub struct TransactionHash(pub [u8; 48]);

impl TransactionHash {
    #[must_use]
    pub(crate) fn new(bytes: &[u8]) -> Self {
        Self(Sha384::digest(bytes).into())
    }
}

impl Debug for TransactionHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for TransactionHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(&hex::encode(self.0))
    }
}
