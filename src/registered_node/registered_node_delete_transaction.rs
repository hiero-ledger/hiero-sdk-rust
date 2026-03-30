// SPDX-License-Identifier: Apache-2.0

use hiero_sdk_proto::services;
use hiero_sdk_proto::services::address_book_service_client::AddressBookServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
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
    ToProtobuf,
    Transaction,
    ValidateChecksums,
};

/// A transaction body to delete a registered node from the network address book.
///
/// This transaction, once complete, SHALL remove the identified registered
/// node from the network state.
/// This transaction MUST be signed by the existing entry `admin_key` or
/// authorized by the Hiero network governance structure.
///
/// HIP-1137
pub type RegisteredNodeDeleteTransaction = Transaction<RegisteredNodeDeleteTransactionData>;

/// A transaction body to delete a registered node from the network address book.
///
/// HIP-1137
#[derive(Debug, Clone, Default)]
pub struct RegisteredNodeDeleteTransactionData {
    /// A registered node identifier in the network state.
    registered_node_id: u64,
}

impl RegisteredNodeDeleteTransaction {
    /// Returns the registered node ID.
    #[must_use]
    pub fn get_registered_node_id(&self) -> u64 {
        self.data().registered_node_id
    }

    /// Sets the registered node ID.
    pub fn registered_node_id(&mut self, registered_node_id: u64) -> &mut Self {
        self.data_mut().registered_node_id = registered_node_id;
        self
    }
}

impl TransactionData for RegisteredNodeDeleteTransactionData {}

impl TransactionExecute for RegisteredNodeDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async {
            AddressBookServiceClient::new(channel)
                .delete_registered_node(request)
                .await
        })
    }
}

impl ValidateChecksums for RegisteredNodeDeleteTransactionData {
    fn validate_checksums(&self, _ledger_id: &RefLedgerId) -> Result<(), Error> {
        Ok(())
    }
}

impl ToTransactionDataProtobuf for RegisteredNodeDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::RegisteredNodeDelete(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for RegisteredNodeDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::RegisteredNodeDelete(self.to_protobuf())
    }
}

impl From<RegisteredNodeDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: RegisteredNodeDeleteTransactionData) -> Self {
        Self::RegisteredNodeDelete(transaction)
    }
}

impl FromProtobuf<services::RegisteredNodeDeleteTransactionBody>
    for RegisteredNodeDeleteTransactionData
{
    fn from_protobuf(pb: services::RegisteredNodeDeleteTransactionBody) -> crate::Result<Self> {
        Ok(Self { registered_node_id: pb.registered_node_id })
    }
}

impl ToProtobuf for RegisteredNodeDeleteTransactionData {
    type Protobuf = services::RegisteredNodeDeleteTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::RegisteredNodeDeleteTransactionBody {
            registered_node_id: self.registered_node_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use hiero_sdk_proto::services;

    use super::RegisteredNodeDeleteTransaction;
    use crate::protobuf::FromProtobuf;
    use crate::registered_node::RegisteredNodeDeleteTransactionData;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::AnyTransaction;

    fn make_transaction() -> RegisteredNodeDeleteTransaction {
        let mut tx = RegisteredNodeDeleteTransaction::new_for_tests();

        tx.registered_node_id(1).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/registered_node_delete_transaction/serialize.txt"]
            .assert_debug_eq(&tx);
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);
        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2)
    }

    #[test]
    fn from_proto_body() {
        let tx = services::RegisteredNodeDeleteTransactionBody { registered_node_id: 1 };

        let data = RegisteredNodeDeleteTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.registered_node_id, 1);
    }

    #[test]
    fn get_set_registered_node_id() {
        let mut tx = RegisteredNodeDeleteTransaction::new();
        tx.registered_node_id(1);

        assert_eq!(tx.get_registered_node_id(), 1);
    }

    #[test]
    #[should_panic]
    fn get_set_registered_node_id_frozen_panic() {
        make_transaction().registered_node_id(1);
    }
}
