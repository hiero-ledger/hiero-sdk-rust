// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
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
    AccountId,
    BoxGrpcFuture,
    ContractId,
    Error,
    Transaction,
    ValidateChecksums,
};

/// Marks a contract as deleted and transfers its remaining hBars, if any, to
/// a designated receiver.
///
pub type ContractDeleteTransaction = Transaction<ContractDeleteTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct ContractDeleteTransactionData {
    contract_id: Option<ContractId>,

    transfer_account_id: Option<AccountId>,

    transfer_contract_id: Option<ContractId>,
}

impl ContractDeleteTransaction {
    /// Returns the ID of the contract that should be deleted.
    #[must_use]
    pub fn get_contract_id(&self) -> Option<ContractId> {
        self.data().contract_id
    }

    /// Sets ID of the contract which should be deleted.
    pub fn contract_id(&mut self, id: ContractId) -> &mut Self {
        self.data_mut().contract_id = Some(id);
        self
    }

    /// Returns the ID of the account which will receive all remaining hbars.
    #[must_use]
    pub fn get_transfer_account_id(&self) -> Option<AccountId> {
        self.data().transfer_account_id
    }

    /// Sets the ID of the account which will receive all remaining hbars.
    pub fn transfer_account_id(&mut self, id: AccountId) -> &mut Self {
        self.data_mut().transfer_account_id = Some(id);
        self
    }

    /// Returns ID of the contract which will receive all rmaining hbars.
    #[must_use]
    pub fn get_transfer_contract_id(&self) -> Option<ContractId> {
        self.data().transfer_contract_id
    }

    /// Sets the the ID of the contract which will receive all remaining hbars.
    pub fn transfer_contract_id(&mut self, id: ContractId) -> &mut Self {
        self.data_mut().transfer_contract_id = Some(id);
        self
    }
}

impl TransactionData for ContractDeleteTransactionData {}

impl TransactionExecute for ContractDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { SmartContractServiceClient::new(channel).delete_contract(request).await })
    }
}

impl ValidateChecksums for ContractDeleteTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.contract_id.validate_checksums(ledger_id)?;
        self.transfer_account_id.validate_checksums(ledger_id)?;
        self.transfer_contract_id.validate_checksums(ledger_id)
    }
}

impl ToTransactionDataProtobuf for ContractDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::ContractDeleteInstance(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for ContractDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::ContractDeleteInstance(self.to_protobuf())
    }
}

impl FromProtobuf<services::ContractDeleteTransactionBody> for ContractDeleteTransactionData {
    fn from_protobuf(pb: services::ContractDeleteTransactionBody) -> crate::Result<Self> {
        use services::contract_delete_transaction_body::Obtainers;

        let (transfer_account_id, transfer_contract_id) = match pb.obtainers {
            Some(Obtainers::TransferAccountId(it)) => (Some(AccountId::from_protobuf(it)?), None),
            Some(Obtainers::TransferContractId(it)) => (None, Some(ContractId::from_protobuf(it)?)),
            None => (None, None),
        };

        Ok(Self {
            contract_id: Option::from_protobuf(pb.contract_id)?,
            transfer_account_id,
            transfer_contract_id,
        })
    }
}

impl From<ContractDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: ContractDeleteTransactionData) -> Self {
        Self::ContractDelete(transaction)
    }
}

impl ToProtobuf for ContractDeleteTransactionData {
    type Protobuf = services::ContractDeleteTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let delete_contract_id = self.contract_id.to_protobuf();

        let obtainers = match (&self.transfer_account_id, &self.transfer_contract_id) {
            (Some(account_id), None) => {
                Some(services::contract_delete_transaction_body::Obtainers::TransferAccountId(
                    account_id.to_protobuf(),
                ))
            }

            (None, Some(contract_id)) => {
                Some(services::contract_delete_transaction_body::Obtainers::TransferContractId(
                    contract_id.to_protobuf(),
                ))
            }

            _ => None,
        };

        services::ContractDeleteTransactionBody {
            contract_id: delete_contract_id,
            permanent_removal: false,
            obtainers,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;

    use crate::contract::ContractDeleteTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AccountId,
        AnyTransaction,
        ContractDeleteTransaction,
        ContractId,
    };

    const CONTRACT_ID: ContractId = ContractId::new(0, 0, 5007);
    const TRANSFER_ACCOUNT_ID: AccountId = AccountId::new(0, 0, 9);
    const TRANSFER_CONTRACT_ID: ContractId = ContractId::new(0, 0, 5008);

    fn make_transaction() -> ContractDeleteTransaction {
        let mut tx = ContractDeleteTransaction::new_for_tests();

        tx.contract_id(CONTRACT_ID)
            .transfer_account_id(TRANSFER_ACCOUNT_ID)
            .transfer_contract_id(TRANSFER_CONTRACT_ID)
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
            ContractDeleteInstance(
                ContractDeleteTransactionBody {
                    contract_id: Some(
                        ContractId {
                            shard_num: 0,
                            realm_num: 0,
                            contract: Some(
                                ContractNum(
                                    5007,
                                ),
                            ),
                        },
                    ),
                    permanent_removal: false,
                    obtainers: None,
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
        let tx = services::ContractDeleteTransactionBody {
            contract_id: Some(CONTRACT_ID.to_protobuf()),
            obtainers: Some(
                services::contract_delete_transaction_body::Obtainers::TransferAccountId(
                    TRANSFER_ACCOUNT_ID.to_protobuf(),
                ),
            ),
            permanent_removal: false,
        };

        let tx = ContractDeleteTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(tx.contract_id, Some(CONTRACT_ID));
        assert_eq!(tx.transfer_account_id, Some(TRANSFER_ACCOUNT_ID));
        assert_eq!(tx.transfer_contract_id, None);
    }

    #[test]
    fn get_set_contract_id() {
        let mut tx = ContractDeleteTransaction::new();
        tx.contract_id(CONTRACT_ID);

        assert_eq!(tx.get_contract_id(), Some(CONTRACT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_contract_id_frozen_panics() {
        make_transaction().contract_id(CONTRACT_ID);
    }

    #[test]
    fn get_set_transfer_account_id() {
        let mut tx = ContractDeleteTransaction::new();
        tx.transfer_account_id(TRANSFER_ACCOUNT_ID);

        assert_eq!(tx.get_transfer_account_id(), Some(TRANSFER_ACCOUNT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_transfer_account_id_frozen_panics() {
        make_transaction().transfer_account_id(TRANSFER_ACCOUNT_ID);
    }

    #[test]
    fn get_set_transfer_contract_id() {
        let mut tx = ContractDeleteTransaction::new();
        tx.transfer_contract_id(TRANSFER_CONTRACT_ID);

        assert_eq!(tx.get_transfer_contract_id(), Some(TRANSFER_CONTRACT_ID));
    }

    #[test]
    #[should_panic]
    fn get_set_transfer_contract_id_frozen_panics() {
        make_transaction().transfer_contract_id(TRANSFER_CONTRACT_ID);
    }
}
