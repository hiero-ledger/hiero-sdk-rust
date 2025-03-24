// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::file_service_client::FileServiceClient;
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
    FileId,
    Transaction,
    ValidateChecksums,
};

/// Delete the given file.
///
/// After deletion, it will be marked as deleted and will have no contents.
/// Information about it will continue to exist until it expires.
///
pub type FileDeleteTransaction = Transaction<FileDeleteTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct FileDeleteTransactionData {
    /// The file to delete. It will be marked as deleted until it expires.
    /// Then it will disappear.
    file_id: Option<FileId>,
}

impl FileDeleteTransaction {
    /// Returns the ID of the file to be deleted.
    #[must_use]
    pub fn get_file_id(&self) -> Option<FileId> {
        self.data().file_id
    }

    /// Sets the ID of the file to be deleted.
    pub fn file_id(&mut self, id: impl Into<FileId>) -> &mut Self {
        self.data_mut().file_id = Some(id.into());
        self
    }
}

impl TransactionData for FileDeleteTransactionData {}

impl TransactionExecute for FileDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { FileServiceClient::new(channel).delete_file(request).await })
    }
}

impl ValidateChecksums for FileDeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.file_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for FileDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::FileDelete(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for FileDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::FileDelete(self.to_protobuf())
    }
}

impl From<FileDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: FileDeleteTransactionData) -> Self {
        Self::FileDelete(transaction)
    }
}

impl FromProtobuf<services::FileDeleteTransactionBody> for FileDeleteTransactionData {
    fn from_protobuf(pb: services::FileDeleteTransactionBody) -> crate::Result<Self> {
        Ok(Self { file_id: Option::from_protobuf(pb.file_id)? })
    }
}

impl ToProtobuf for FileDeleteTransactionData {
    type Protobuf = services::FileDeleteTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::FileDeleteTransactionBody { file_id: self.file_id.to_protobuf() }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use crate::file::FileDeleteTransactionData;
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
        FileDeleteTransaction,
        FileId,
    };

    const FILE_ID: FileId = FileId::new(0, 0, 6006);

    fn make_transaction() -> FileDeleteTransaction {
        let mut tx = FileDeleteTransaction::new_for_tests();

        tx.file_id(FILE_ID).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            FileDelete(
                FileDeleteTransactionBody {
                    file_id: Some(
                        FileId {
                            shard_num: 0,
                            realm_num: 0,
                            file_num: 6006,
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
        let tx = services::FileDeleteTransactionBody { file_id: Some(FILE_ID.to_protobuf()) };

        let tx = FileDeleteTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.file_id, Some(FILE_ID));
    }

    #[test]
    fn get_set_file_id() {
        let mut tx = FileDeleteTransaction::new();
        tx.file_id(FILE_ID);

        assert_eq!(tx.get_file_id(), Some(FILE_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_file_id_frozen_panics() {
        make_transaction().file_id(FILE_ID);
    }
}
