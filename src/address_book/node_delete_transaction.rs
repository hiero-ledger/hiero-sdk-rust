// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::address_book_service_client::AddressBookServiceClient;
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

/// A transaction body to delete a node from the network address book.
///
/// This transaction body SHALL be considered a "privileged transaction".
///
/// - A `NodeDeleteTransactionBody` MUST be signed by the governing council.
/// - Upon success, the address book entry SHALL enter a "pending delete"
///    state.
/// - All address book entries pending deletion SHALL be removed from the
///    active network configuration during the next `freeze` transaction with
///    the field `freeze_type` set to `PREPARE_UPGRADE`.<br/>
/// - A deleted address book node SHALL be removed entirely from network state.
/// - A deleted address book node identifier SHALL NOT be reused.
///
/// ### Record Stream Effects
/// Upon completion the "deleted" `node_id` SHALL be in the transaction
/// receipt.
pub type NodeDeleteTransaction = Transaction<NodeDeleteTransactionData>;

/// A transaction body to delete a node from the network address book.
#[derive(Debug, Clone, Default)]
pub struct NodeDeleteTransactionData {
    /// A consensus node identifier in the network state.
    node_id: u64,
}

impl NodeDeleteTransaction {
    /// Returns the node ID associated with the node to be deleted.
    #[must_use]
    pub fn get_node_id(&self) -> u64 {
        self.data().node_id
    }

    /// Sets the node ID associated with the node to be deleted.
    pub fn node_id(&mut self, node_id: u64) -> &mut Self {
        self.data_mut().node_id = node_id;
        self
    }
}

impl TransactionData for NodeDeleteTransactionData {}

impl TransactionExecute for NodeDeleteTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { AddressBookServiceClient::new(channel).delete_node(request).await })
    }
}

impl ValidateChecksums for NodeDeleteTransactionData {
    fn validate_checksums(&self, _ledger_id: &RefLedgerId) -> Result<(), Error> {
        Ok(())
    }
}

impl ToTransactionDataProtobuf for NodeDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::NodeDelete(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for NodeDeleteTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::NodeDelete(self.to_protobuf())
    }
}

impl From<NodeDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: NodeDeleteTransactionData) -> Self {
        Self::NodeDelete(transaction)
    }
}

impl FromProtobuf<services::NodeDeleteTransactionBody> for NodeDeleteTransactionData {
    fn from_protobuf(pb: services::NodeDeleteTransactionBody) -> crate::Result<Self> {
        Ok(Self { node_id: pb.node_id })
    }
}

impl ToProtobuf for NodeDeleteTransactionData {
    type Protobuf = services::NodeDeleteTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::NodeDeleteTransactionBody { node_id: self.node_id }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use hedera_proto::services;

    use super::NodeDeleteTransaction;
    use crate::address_book::NodeDeleteTransactionData;
    use crate::protobuf::FromProtobuf;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::AnyTransaction;

    fn make_transaction() -> NodeDeleteTransaction {
        let mut tx = NodeDeleteTransaction::new_for_tests();

        tx.node_id(1).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/node_delete_transaction/serialize.txt"].assert_debug_eq(&tx);
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
        let tx = services::NodeDeleteTransactionBody { node_id: 1 };

        let data = NodeDeleteTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.node_id, 1);
    }

    #[test]
    fn get_set_node_id() {
        let mut tx = NodeDeleteTransaction::new();
        tx.node_id(1);

        assert_eq!(tx.get_node_id(), 1);
    }

    #[test]
    #[should_panic]
    fn get_set_node_id_frozen_panic() {
        make_transaction().node_id(1);
    }
}
