// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::schedule_service_client::ScheduleServiceClient;
use tonic::transport::Channel;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    BoxGrpcFuture,
    Error,
    ScheduleId,
    Transaction,
    ValidateChecksums,
};

/// Adds zero or more signing keys to a schedule.
pub type ScheduleSignTransaction = Transaction<ScheduleSignTransactionData>;

#[derive(Debug, Default, Clone)]
pub struct ScheduleSignTransactionData {
    schedule_id: Option<ScheduleId>,
}

impl ScheduleSignTransaction {
    /// Returns the schedule to add signing keys to.
    #[must_use]
    pub fn get_schedule_id(&self) -> Option<ScheduleId> {
        self.data().schedule_id
    }

    /// Sets the schedule to add signing keys to.
    pub fn schedule_id(&mut self, id: ScheduleId) -> &mut Self {
        self.data_mut().schedule_id = Some(id);
        self
    }
}

impl TransactionData for ScheduleSignTransactionData {}

impl TransactionExecute for ScheduleSignTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { ScheduleServiceClient::new(channel).delete_schedule(request).await })
    }
}

impl ValidateChecksums for ScheduleSignTransactionData {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        self.schedule_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for ScheduleSignTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        let schedule_id = self.schedule_id.to_protobuf();

        services::transaction_body::Data::ScheduleSign(services::ScheduleSignTransactionBody {
            schedule_id,
        })
    }
}

impl From<ScheduleSignTransactionData> for AnyTransactionData {
    fn from(transaction: ScheduleSignTransactionData) -> Self {
        Self::ScheduleSign(transaction)
    }
}

impl FromProtobuf<services::ScheduleSignTransactionBody> for ScheduleSignTransactionData {
    fn from_protobuf(pb: services::ScheduleSignTransactionBody) -> crate::Result<Self> {
        Ok(Self { schedule_id: Option::from_protobuf(pb.schedule_id)? })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::schedule::ScheduleSignTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyTransaction,
        ScheduleId,
        ScheduleSignTransaction,
    };

    const SCHEDULE_ID: ScheduleId = ScheduleId::new(0, 0, 444);

    fn make_transaction() -> ScheduleSignTransaction {
        let mut tx = ScheduleSignTransaction::new_for_tests();

        tx.schedule_id(SCHEDULE_ID).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            ScheduleSign(
                ScheduleSignTransactionBody {
                    schedule_id: Some(
                        ScheduleId {
                            shard_num: 0,
                            realm_num: 0,
                            schedule_num: 444,
                        },
                    ),
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn from_proto_body() {
        let tx =
            services::ScheduleSignTransactionBody { schedule_id: Some(SCHEDULE_ID.to_protobuf()) };

        let tx = ScheduleSignTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.schedule_id, Some(SCHEDULE_ID));
    }

    #[test]
    fn get_set_schedule_id() {
        let mut tx = ScheduleSignTransaction::new();
        tx.schedule_id(SCHEDULE_ID);

        assert_eq!(tx.get_schedule_id(), Some(SCHEDULE_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_schedule_id_frozen_panics() {
        make_transaction().schedule_id(SCHEDULE_ID);
    }
}
