// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::freeze_service_client::FreezeServiceClient;
use time::OffsetDateTime;
use tonic::transport::Channel;

use crate::protobuf::FromProtobuf;
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
    FileId,
    FreezeType,
    ToProtobuf,
    Transaction,
    ValidateChecksums,
};

/// Sets the freezing period in which the platform will stop creating
/// events and accepting transactions.
///
/// This is used before safely shut down the platform for maintenance.
///
pub type FreezeTransaction = Transaction<FreezeTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct FreezeTransactionData {
    start_time: Option<OffsetDateTime>,
    file_id: Option<FileId>,
    file_hash: Option<Vec<u8>>,
    freeze_type: FreezeType,
}

impl FreezeTransaction {
    /// Returns the start time.
    #[must_use]
    pub fn get_start_time(&self) -> Option<OffsetDateTime> {
        self.data().start_time
    }

    /// Sets the start time.
    pub fn start_time(&mut self, time: OffsetDateTime) -> &mut Self {
        self.data_mut().start_time = Some(time);
        self
    }

    /// Returns the freeze type.
    #[must_use]
    pub fn get_freeze_type(&self) -> FreezeType {
        self.data().freeze_type
    }

    /// Sets the freeze type.
    pub fn freeze_type(&mut self, ty: FreezeType) -> &mut Self {
        self.data_mut().freeze_type = ty;
        self
    }

    /// Returns the file ID.
    #[must_use]
    pub fn get_file_id(&self) -> Option<FileId> {
        self.data().file_id
    }

    /// Sets the file ID.
    pub fn file_id(&mut self, id: FileId) -> &mut Self {
        self.data_mut().file_id = Some(id);
        self
    }

    /// Returns the file hash.
    #[must_use]
    pub fn get_file_hash(&self) -> Option<&[u8]> {
        self.data().file_hash.as_deref()
    }

    /// Sets the file hash.
    pub fn file_hash(&mut self, hash: Vec<u8>) -> &mut Self {
        self.data_mut().file_hash = Some(hash);
        self
    }
}

impl TransactionData for FreezeTransactionData {}

impl TransactionExecute for FreezeTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { FreezeServiceClient::new(channel).freeze(request).await })
    }
}

impl ValidateChecksums for FreezeTransactionData {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        self.file_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for FreezeTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::Freeze(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for FreezeTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::Freeze(self.to_protobuf())
    }
}

impl From<FreezeTransactionData> for AnyTransactionData {
    fn from(transaction: FreezeTransactionData) -> Self {
        Self::Freeze(transaction)
    }
}

impl FromProtobuf<services::FreezeTransactionBody> for FreezeTransactionData {
    fn from_protobuf(pb: services::FreezeTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            start_time: pb.start_time.map(Into::into),
            file_id: Option::from_protobuf(pb.update_file)?,
            file_hash: Some(pb.file_hash),
            freeze_type: FreezeType::from(pb.freeze_type),
        })
    }
}

impl ToProtobuf for FreezeTransactionData {
    type Protobuf = services::FreezeTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::FreezeTransactionBody {
            update_file: self.file_id.to_protobuf(),
            file_hash: self.file_hash.clone().unwrap_or_default(),
            start_time: self.start_time.map(Into::into),
            freeze_type: self.freeze_type as _,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use hex_literal::hex;
    use time::OffsetDateTime;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::system::FreezeTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        VALID_START,
    };
    use crate::{
        AnyTransaction,
        FileId,
        FreezeTransaction,
        FreezeType,
    };

    const FILE_ID: FileId = FileId::new(4, 5, 6);
    const FILE_HASH: [u8; 14] = hex!("1723904587120938954702349857");
    const START_TIME: OffsetDateTime = VALID_START;
    const FREEZE_TYPE: FreezeType = FreezeType::FreezeAbort;

    fn make_transaction() -> FreezeTransaction {
        let mut tx = FreezeTransaction::new_for_tests();

        tx.file_id(FILE_ID)
            .file_hash(FILE_HASH.to_vec())
            .start_time(START_TIME)
            .freeze_type(FREEZE_TYPE)
            .freeze()
            .unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            Freeze(
                FreezeTransactionBody {
                    start_hour: 0,
                    start_min: 0,
                    end_hour: 0,
                    end_min: 0,
                    update_file: Some(
                        FileId {
                            shard_num: 4,
                            realm_num: 5,
                            file_num: 6,
                        },
                    ),
                    file_hash: [
                        23,
                        35,
                        144,
                        69,
                        135,
                        18,
                        9,
                        56,
                        149,
                        71,
                        2,
                        52,
                        152,
                        87,
                    ],
                    start_time: Some(
                        Timestamp {
                            seconds: 1554158542,
                            nanos: 0,
                        },
                    ),
                    freeze_type: FreezeAbort,
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
        let tx = services::FreezeTransactionBody {
            update_file: Some(FILE_ID.to_protobuf()),
            file_hash: FILE_HASH.to_vec(),
            start_time: Some(START_TIME.to_protobuf()),
            freeze_type: FREEZE_TYPE as i32,
            ..Default::default()
        };

        let tx = FreezeTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.file_id, Some(FILE_ID));
        assert_eq!(tx.file_hash.as_deref(), Some(FILE_HASH.as_slice()));
        assert_eq!(tx.start_time, Some(START_TIME));
        assert_eq!(tx.freeze_type, FREEZE_TYPE);
    }

    mod get_set {
        use super::*;

        #[test]
        fn file_id() {
            let mut tx = FreezeTransaction::new();
            tx.file_id(FILE_ID);

            assert_eq!(tx.get_file_id(), Some(FILE_ID));
        }

        #[test]
        #[should_panic]
        fn file_id_frozen_panics() {
            make_transaction().file_id(FILE_ID);
        }

        #[test]
        fn file_hash() {
            let mut tx = FreezeTransaction::new();
            tx.file_hash(FILE_HASH.to_vec());

            assert_eq!(tx.get_file_hash(), Some(FILE_HASH.as_slice()));
        }

        #[test]
        #[should_panic]
        fn file_hash_frozen_panics() {
            make_transaction().file_hash(FILE_HASH.to_vec());
        }

        #[test]
        fn start_time() {
            let mut tx = FreezeTransaction::new();
            tx.start_time(START_TIME);

            assert_eq!(tx.get_start_time(), Some(START_TIME));
        }

        #[test]
        #[should_panic]
        fn start_time_frozen_panics() {
            make_transaction().start_time(START_TIME);
        }

        #[test]
        fn freeze_type() {
            let mut tx = FreezeTransaction::new();
            tx.freeze_type(FREEZE_TYPE);

            assert_eq!(tx.get_freeze_type(), FREEZE_TYPE);
        }

        #[test]
        #[should_panic]
        fn freeze_type_frozen_panics() {
            make_transaction().freeze_type(FREEZE_TYPE);
        }
    }
}
