// SPDX-License-Identifier: Apache-2.0

// not happy about needing this.
/// Downcast from one type to another.
pub trait DowncastOwned<T>: Sized {
    fn downcast_owned(self) -> Result<T, Self>;
}
