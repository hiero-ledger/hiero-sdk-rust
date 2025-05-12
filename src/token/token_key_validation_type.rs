// SPDX-License-Identifier: Apache-2.0

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};

use hedera_proto::services;

use crate::{
    FromProtobuf,
    ToProtobuf,
};

/// Types of validation strategies for token keys.
/// Defaults to [`FullValidation`](Self::FullValidation).
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
#[repr(C)]
pub enum TokenKeyValidation {
    /// Currently the default behaviour. It will perform all token key validations.
    #[default]
    FullValidation = 0,

    /// Perform no validations at all for all passed token keys.
    NoValidation = 1,
}

impl Display for TokenKeyValidation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FullValidation => write!(f, "FULL_VALIDATION"),
            Self::NoValidation => write!(f, "NO_VALIDATION"),
        }
    }
}

impl FromProtobuf<services::TokenKeyValidation> for TokenKeyValidation {
    fn from_protobuf(pb: services::TokenKeyValidation) -> crate::Result<Self> {
        Ok(match pb {
            services::TokenKeyValidation::FullValidation => Self::FullValidation,
            services::TokenKeyValidation::NoValidation => Self::NoValidation,
        })
    }
}

impl ToProtobuf for TokenKeyValidation {
    type Protobuf = services::TokenKeyValidation;

    fn to_protobuf(&self) -> Self::Protobuf {
        match self {
            Self::FullValidation => Self::Protobuf::FullValidation,
            Self::NoValidation => Self::Protobuf::NoValidation,
        }
    }
}
