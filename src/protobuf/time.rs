// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use time::{
    Duration,
    OffsetDateTime,
};

use crate::ToProtobuf;

impl ToProtobuf for Duration {
    type Protobuf = services::Duration;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::Duration { seconds: self.whole_seconds() }
    }
}

impl ToProtobuf for OffsetDateTime {
    type Protobuf = services::Timestamp;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::Timestamp { seconds: self.unix_timestamp(), nanos: self.nanosecond() as i32 }
    }
}
