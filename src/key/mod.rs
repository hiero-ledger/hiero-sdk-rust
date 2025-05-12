// SPDX-License-Identifier: Apache-2.0

#[allow(clippy::module_inception)]
mod key;
mod key_list;
mod private_key;
mod public_key;

pub use key::Key;
pub use key_list::KeyList;
pub use private_key::PrivateKey;
pub use public_key::PublicKey;

#[derive(Copy, Clone, Debug)]
pub(crate) enum KeyKind {
    Ed25519,
    Ecdsa,
}
