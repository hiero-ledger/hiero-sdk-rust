// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use triomphe::Arc;
use unsize::{
    CoerceUnsize,
    Coercion,
};

use crate::{
    PrivateKey,
    PublicKey,
};

#[derive(Clone)]
pub(crate) enum AnySigner {
    PrivateKey(PrivateKey),
    // public key is 216 bytes.
    // Here be a story of dragons.
    // Once an engineer attempted to downgrade this `Arc` to a mere `Box`, alas it was not meant to be.
    // For the Fn must be cloned, and `dyn Fn` must not.
    // The plan to not pay the price of Arc was doomed from the very beginning.
    // Attempts to avoid the arc, the cloning of the `Fn`, all end in misery,
    // for the `Client` must have `AnySigner`, not a `PrivateKey`, and the `ContractCreateFlow`...
    // Well, it must be executable multiple times, for ownership reasons.
    // lint note: can't reasonably resolve this because putting the `type` on anything but the `Fn(..)` part is useless
    // but we can't do that because trait aliases don't exist.
    #[allow(clippy::type_complexity)]
    Arbitrary(Box<PublicKey>, Arc<dyn Fn(&[u8]) -> Vec<u8> + Send + Sync>),
}

impl AnySigner {
    pub(crate) fn arbitrary<F: Fn(&[u8]) -> Vec<u8> + Send + Sync + 'static>(
        public_key: Box<PublicKey>,
        signer: F,
    ) -> Self {
        Self::Arbitrary(
            public_key,
            Arc::new(signer).unsize(Coercion!(to dyn Fn(&[u8]) -> Vec<u8> + Send + Sync)),
        )
    }
}

impl fmt::Debug for AnySigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrivateKey(_) => f.debug_tuple("PrivateKey").field(&"..").finish(),
            Self::Arbitrary(arg0, _) => {
                f.debug_tuple("Arbitrary").field(arg0).field(&"Fn").finish()
            }
        }
    }
}

impl AnySigner {
    // *Cheap* Accessor to get the public key without signing the message first.
    pub(crate) fn public_key(&self) -> PublicKey {
        match self {
            AnySigner::PrivateKey(it) => it.public_key(),
            AnySigner::Arbitrary(it, _) => **it,
        }
    }

    pub(crate) fn sign(&self, message: &[u8]) -> (PublicKey, Vec<u8>) {
        match self {
            AnySigner::PrivateKey(it) => (it.public_key(), it.sign(message)),
            AnySigner::Arbitrary(public, signer) => {
                let bytes = signer(message);

                (**public, bytes)
            }
        }
    }
}
