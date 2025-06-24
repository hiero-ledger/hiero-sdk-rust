// SPDX-License-Identifier: Apache-2.0

#![allow(non_camel_case_types)]
#![allow(clippy::default_trait_access, clippy::doc_markdown)]

#[cfg(feature = "time_0_3")]
mod time_0_3;

#[cfg(feature = "fraction")]
mod fraction;

// fixme: Do this, just, don't warn 70 times in generated code.
#[allow(clippy::derive_partial_eq_without_eq)]
pub mod services {
    tonic::include_proto!("proto");
}

// fixme: Do this, just, don't warn 70 times in generated code.
#[allow(clippy::derive_partial_eq_without_eq)]
pub mod mirror {
    tonic::include_proto!("mirror/com.hedera.mirror.api.proto");
}

// fixme: Do this, just, don't warn 70 times in generated code.
#[allow(clippy::derive_partial_eq_without_eq)]
pub mod sdk {
    tonic::include_proto!("sdk/proto");
}
