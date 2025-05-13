// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::schedule_service_client::ScheduleServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
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

/// Marks a schedule in the network's action queue as deleted. Must be signed
/// by the admin key of the target schedule. A deleted schedule cannot
/// receive any additional signing keys, nor will it be executed.
pub type ScheduleDeleteTransaction = Transaction<ScheduleDeleteTransactionData>;

#[derive(Debug, Default, Clone)]
pub struct ScheduleDeleteTransactionData {
    schedule_id: Option<ScheduleId>,
}

impl ScheduleDeleteTransaction {
    /// Returns the schedule to delete.
    #[must_use]
    pub fn get_schedule_id(&self) -> Option<ScheduleId> {
        self.data().schedule_id
    }

    /// Sets the schedule to delete.
    pub fn schedule_id(&mut self, id: ScheduleId) -> &mut Self {
        self.data_mut().schedule_id = Some(id);
        self
    }
}
impl TransactionData for ScheduleDeleteTransactionData {}

impl TransactionExecute for ScheduleDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { ScheduleServiceClient::new(channel).delete_schedule(request).await })
    }
}

impl ValidateChecksums for ScheduleDeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.schedule_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for ScheduleDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::ScheduleDelete(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for ScheduleDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::ScheduleDelete(self.to_protobuf())
    }
}

impl From<ScheduleDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: ScheduleDeleteTransactionData) -> Self {
        Self::ScheduleDelete(transaction)
    }
}

impl FromProtobuf<services::ScheduleDeleteTransactionBody> for ScheduleDeleteTransactionData {
    fn from_protobuf(pb: services::ScheduleDeleteTransactionBody) -> crate::Result<Self> {
        Ok(Self { schedule_id: Option::from_protobuf(pb.schedule_id)? })
    }
}

impl ToProtobuf for ScheduleDeleteTransactionData {
    type Protobuf = services::ScheduleDeleteTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::ScheduleDeleteTransactionBody { schedule_id: self.schedule_id.to_protobuf() }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use super::ScheduleDeleteTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyTransaction,
        ScheduleDeleteTransaction,
        ScheduleId,
    };

    const SCHEDULE_ID: ScheduleId = ScheduleId::new(0, 0, 444);

    fn make_transaction() -> ScheduleDeleteTransaction {
        let mut tx = ScheduleDeleteTransaction::new_for_tests();

        tx.schedule_id(SCHEDULE_ID).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            ScheduleDelete(
                ScheduleDeleteTransactionBody {
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
        let tx = services::ScheduleDeleteTransactionBody {
            schedule_id: Some(SCHEDULE_ID.to_protobuf()),
        };

        let tx = ScheduleDeleteTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.schedule_id, Some(SCHEDULE_ID));
    }

    #[test]
    fn get_set_schedule_id() {
        let mut tx = ScheduleDeleteTransaction::new();
        tx.schedule_id(SCHEDULE_ID);

        assert_eq!(tx.get_schedule_id(), Some(SCHEDULE_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_schedule_id_frozen_panics() {
        make_transaction().schedule_id(SCHEDULE_ID);
    }
}
