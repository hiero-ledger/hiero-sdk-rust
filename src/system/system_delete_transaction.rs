// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::file_service_client::FileServiceClient;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use time::OffsetDateTime;
use tonic::transport::Channel;

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
    ContractId,
    Error,
    FileId,
    Transaction,
    ValidateChecksums,
};

/// Delete a file or smart contract - can only be done by a Hiero admin.
pub type SystemDeleteTransaction = Transaction<SystemDeleteTransactionData>;

/// Delete a file or smart contract - can only be done by a Hiero admin.
///
/// When it is deleted, it immediately disappears from the system as seen by the user,
/// but is still stored internally until the expiration time, at which time it
/// is truly and permanently deleted.
///
/// Until that time, it can be undeleted by the Hiero admin.
/// When a smart contract is deleted, the cryptocurrency account within it continues
/// to exist, and is not affected by the expiration time here.
///

#[derive(Debug, Clone, Default)]
pub struct SystemDeleteTransactionData {
    expiration_time: Option<OffsetDateTime>,
    file_id: Option<FileId>,
    contract_id: Option<ContractId>,
}

impl SystemDeleteTransaction {
    /// Returns the contract ID which should be deleted.
    #[must_use]
    pub fn get_contract_id(&self) -> Option<ContractId> {
        self.data().contract_id
    }

    /// Sets the contract ID which should be deleted.
    pub fn contract_id(&mut self, id: impl Into<ContractId>) -> &mut Self {
        let data = self.data_mut();
        data.file_id = None;
        data.contract_id = Some(id.into());
        self
    }

    /// Returns the file ID which should be deleted.
    #[must_use]
    pub fn get_file_id(&self) -> Option<FileId> {
        self.data().file_id
    }

    /// Sets the file ID which should be deleted.
    pub fn file_id(&mut self, id: impl Into<FileId>) -> &mut Self {
        let data = self.data_mut();
        data.contract_id = None;
        data.file_id = Some(id.into());
        self
    }

    /// Returns the timestamp at which the "deleted" entity should
    /// truly be permanently deleted.
    #[must_use]
    pub fn get_expiration_time(&self) -> Option<OffsetDateTime> {
        self.data().expiration_time
    }

    /// Sets the timestamp at which the "deleted" file should
    /// truly be permanently deleted.
    pub fn expiration_time(&mut self, expiration_time: OffsetDateTime) -> &mut Self {
        self.data_mut().expiration_time = Some(expiration_time);
        self
    }
}

impl TransactionData for SystemDeleteTransactionData {}

impl TransactionExecute for SystemDeleteTransactionData {
    #[allow(deprecated)]
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async move {
            if self.file_id.is_some() {
                FileServiceClient::new(channel).system_delete(request).await
            } else {
                SmartContractServiceClient::new(channel).system_delete(request).await
            }
        })
    }
}

impl ValidateChecksums for SystemDeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &crate::ledger_id::RefLedgerId) -> Result<(), Error> {
        self.file_id.validate_checksums(ledger_id)?;
        self.contract_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for SystemDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::SystemDelete(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for SystemDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::SystemDelete(self.to_protobuf())
    }
}

impl From<SystemDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: SystemDeleteTransactionData) -> Self {
        Self::SystemDelete(transaction)
    }
}

impl FromProtobuf<services::SystemDeleteTransactionBody> for SystemDeleteTransactionData {
    fn from_protobuf(pb: services::SystemDeleteTransactionBody) -> crate::Result<Self> {
        use services::system_delete_transaction_body::Id;
        let (file_id, contract_id) = match pb.id {
            Some(Id::FileId(it)) => (Some(FileId::from_protobuf(it)?), None),
            Some(Id::ContractId(it)) => (None, Some(ContractId::from_protobuf(it)?)),
            None => (None, None),
        };

        Ok(Self { file_id, contract_id, expiration_time: pb.expiration_time.map(Into::into) })
    }
}

impl ToProtobuf for SystemDeleteTransactionData {
    type Protobuf = services::SystemDeleteTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let expiration_time = self.expiration_time.map(Into::into);
        let contract_id = self.contract_id.to_protobuf();
        let file_id = self.file_id.to_protobuf();

        let id = match (contract_id, file_id) {
            (Some(contract_id), _) => {
                Some(services::system_delete_transaction_body::Id::ContractId(contract_id))
            }

            (_, Some(file_id)) => {
                Some(services::system_delete_transaction_body::Id::FileId(file_id))
            }

            _ => None,
        };

        services::SystemDeleteTransactionBody { expiration_time, id }
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
    use crate::system::SystemDeleteTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        VALID_START,
    };
    use crate::{
        AnyTransaction,
        ContractId,
        FileId,
        SystemDeleteTransaction,
    };

    const FILE_ID: FileId = FileId::new(0, 0, 444);
    const CONTRACT_ID: ContractId = ContractId::new(0, 0, 444);

    fn make_transaction_file() -> SystemDeleteTransaction {
        let mut tx = SystemDeleteTransaction::new_for_tests();

        tx.file_id(FILE_ID).expiration_time(VALID_START).freeze().unwrap();
        tx
    }

    fn make_transaction_contract() -> SystemDeleteTransaction {
        let mut tx = SystemDeleteTransaction::new_for_tests();

        tx.contract_id(CONTRACT_ID).expiration_time(VALID_START).freeze().unwrap();
        tx
    }

    #[test]
    fn serialize_file() {
        let tx = make_transaction_file();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            SystemDelete(
                SystemDeleteTransactionBody {
                    expiration_time: Some(
                        TimestampSeconds {
                            seconds: 1554158542,
                        },
                    ),
                    id: Some(
                        FileId(
                            FileId {
                                shard_num: 0,
                                realm_num: 0,
                                file_num: 444,
                            },
                        ),
                    ),
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn serialize_contract() {
        let tx = make_transaction_contract();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            SystemDelete(
                SystemDeleteTransactionBody {
                    expiration_time: Some(
                        TimestampSeconds {
                            seconds: 1554158542,
                        },
                    ),
                    id: Some(
                        ContractId(
                            ContractId {
                                shard_num: 0,
                                realm_num: 0,
                                contract: Some(
                                    ContractNum(
                                        444,
                                    ),
                                ),
                            },
                        ),
                    ),
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes_file() {
        let tx = make_transaction_file();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn to_from_bytes_contract() {
        let tx = make_transaction_contract();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn from_proto_body() {
        let tx = services::SystemDeleteTransactionBody {
            expiration_time: Some(services::TimestampSeconds {
                seconds: VALID_START.unix_timestamp(),
            }),
            id: Some(services::system_delete_transaction_body::Id::FileId(FILE_ID.to_protobuf())),
        };

        let tx = SystemDeleteTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.file_id, Some(FILE_ID));
        assert_eq!(tx.contract_id, None);
        assert_eq!(tx.expiration_time, Some(VALID_START));
    }

    #[test]
    fn get_set_file_id() {
        let mut tx = SystemDeleteTransaction::new();
        tx.file_id(FILE_ID);

        assert_eq!(tx.get_file_id(), Some(FILE_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_file_id_frozen_panics() {
        make_transaction_file().file_id(FILE_ID);
    }

    #[test]
    fn get_set_contract_id() {
        let mut tx = SystemDeleteTransaction::new();
        tx.contract_id(CONTRACT_ID);

        assert_eq!(tx.get_contract_id(), Some(CONTRACT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_contract_id_frozen_panics() {
        make_transaction_file().contract_id(CONTRACT_ID);
    }

    #[test]
    fn get_set_expiration_time() {
        let mut tx = SystemDeleteTransaction::new();
        tx.expiration_time(VALID_START);

        assert_eq!(tx.get_expiration_time(), Some(VALID_START));
    }

    #[test]
    #[should_panic]
    fn get_set_expiration_time_frozen_panics() {
        make_transaction_file().expiration_time(VALID_START);
    }
}
