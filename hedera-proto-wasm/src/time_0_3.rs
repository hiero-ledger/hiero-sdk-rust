// SPDX-License-Identifier: Apache-2.0

use time::{
    Duration,
    OffsetDateTime,
};

impl From<super::proto::proto::Duration> for Duration {
    fn from(pb: super::proto::proto::Duration) -> Self {
        Self::seconds(pb.seconds)
    }
}

impl From<Duration> for super::proto::proto::Duration {
    fn from(duration: Duration) -> Self {
        Self { seconds: duration.whole_seconds() }
    }
}

impl From<super::proto::proto::TimestampSeconds> for OffsetDateTime {
    fn from(pb: super::proto::proto::TimestampSeconds) -> Self {
        OffsetDateTime::from_unix_timestamp(pb.seconds).unwrap()
    }
}

impl From<OffsetDateTime> for super::proto::proto::TimestampSeconds {
    fn from(dt: OffsetDateTime) -> Self {
        Self { seconds: dt.unix_timestamp() }
    }
}

impl From<super::proto::proto::Timestamp> for OffsetDateTime {
    fn from(pb: super::proto::proto::Timestamp) -> Self {
        OffsetDateTime::from_unix_timestamp(pb.seconds).unwrap()
            + Duration::nanoseconds(pb.nanos.into())
    }
}

impl From<OffsetDateTime> for super::proto::proto::Timestamp {
    fn from(dt: OffsetDateTime) -> Self {
        Self { seconds: dt.unix_timestamp(), nanos: dt.nanosecond() as i32 }
    }
}

