// SPDX-License-Identifier: Apache-2.0

mod any;
mod subscribe;

pub(crate) use any::AnyMirrorQueryData;
pub use any::{
    AnyMirrorQuery,
    AnyMirrorQueryMessage,
    AnyMirrorQueryResponse,
};
pub(crate) use subscribe::{
    subscribe,
    MirrorRequest,
};

use self::subscribe::MirrorQueryExecute;

/// A query that can be executed on the Hedera mirror network.
#[derive(Clone, Debug, Default)]
pub struct MirrorQuery<D> {
    pub(crate) data: D,
    // Field needs to exist even though it currently does nothing
    #[allow(dead_code)]
    pub(crate) common: MirrorQueryCommon,
}

// intentionally inaccessable despite publicity.
#[derive(Clone, Debug, Default)]
pub struct MirrorQueryCommon {
    // empty for now
    // TODO: request_timeout
}

impl<D> MirrorQuery<D>
where
    D: MirrorQueryExecute + Default,
{
    /// Create a new query ready for configuration and execution.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
