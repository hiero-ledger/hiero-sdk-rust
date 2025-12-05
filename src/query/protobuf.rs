// SPDX-License-Identifier: Apache-2.0

use hiero_sdk_proto::services;

pub trait ToQueryProtobuf: Send + Sync {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query;
}
