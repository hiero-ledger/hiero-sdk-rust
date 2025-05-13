// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::schedule_service_client::ScheduleServiceClient;
use tonic::transport::Channel;

use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    BoxGrpcFuture,
    Error,
    Query,
    ScheduleId,
    ScheduleInfo,
    ToProtobuf,
    ValidateChecksums,
};

/// Get all the information about a schedule.
pub type ScheduleInfoQuery = Query<ScheduleInfoQueryData>;

#[derive(Default, Clone, Debug)]
pub struct ScheduleInfoQueryData {
    schedule_id: Option<ScheduleId>,
}

impl From<ScheduleInfoQueryData> for AnyQueryData {
    #[inline]
    fn from(data: ScheduleInfoQueryData) -> Self {
        Self::ScheduleInfo(data)
    }
}

impl ScheduleInfoQuery {
    /// Returns the schedule ID for which information is requested.
    #[must_use]
    pub fn get_schedule_id(&self) -> Option<ScheduleId> {
        self.data.schedule_id
    }

    /// Sets the schedule ID for which information is requested.
    pub fn schedule_id(&mut self, id: impl Into<ScheduleId>) -> &mut Self {
        self.data.schedule_id = Some(id.into());
        self
    }
}

impl ToQueryProtobuf for ScheduleInfoQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let schedule_id = self.schedule_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::ScheduleGetInfo(services::ScheduleGetInfoQuery {
                schedule_id,
                header: Some(header),
            })),
        }
    }
}

impl QueryExecute for ScheduleInfoQueryData {
    type Response = ScheduleInfo;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { ScheduleServiceClient::new(channel).get_schedule_info(request).await })
    }
}

impl ValidateChecksums for ScheduleInfoQueryData {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        self.schedule_id.validate_checksums(ledger_id)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::query::ToQueryProtobuf;
    use crate::{
        Hbar,
        ScheduleId,
        ScheduleInfoQuery,
    };

    #[test]
    fn serialize() {
        expect![[r#"
            Query {
                query: Some(
                    ScheduleGetInfo(
                        ScheduleGetInfoQuery {
                            header: Some(
                                QueryHeader {
                                    payment: None,
                                    response_type: AnswerOnly,
                                },
                            ),
                            schedule_id: Some(
                                ScheduleId {
                                    shard_num: 0,
                                    realm_num: 0,
                                    schedule_num: 5005,
                                },
                            ),
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(
            &ScheduleInfoQuery::new()
                .schedule_id(ScheduleId::new(0, 0, 5005))
                .max_payment_amount(Hbar::from_tinybars(100_000))
                .data
                .to_query_protobuf(Default::default()),
        )
    }

    #[test]
    fn get_set_schedule_id() {
        let mut query = ScheduleInfoQuery::new();
        query.schedule_id(ScheduleId::new(0, 0, 5005));

        assert_eq!(query.get_schedule_id(), Some(ScheduleId::new(0, 0, 5005)));
    }
}
