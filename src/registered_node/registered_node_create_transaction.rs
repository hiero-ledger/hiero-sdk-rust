// SPDX-License-Identifier: Apache-2.0

use hiero_sdk_proto::services;
use hiero_sdk_proto::services::address_book_service_client::AddressBookServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::FromProtobuf;
use crate::registered_service_endpoint::RegisteredServiceEndpoint;
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
    Key,
    ToProtobuf,
    Transaction,
    ValidateChecksums,
};

/// A transaction body to create a new registered node in the network address book.
///
/// This transaction, once complete, SHALL add a new registered node to the
/// network state. The new registered node SHALL be visible and discoverable
/// upon completion of this transaction.
///
/// HIP-1137
pub type RegisteredNodeCreateTransaction = Transaction<RegisteredNodeCreateTransactionData>;

/// A transaction body to create a new registered node in the network address book.
///
/// HIP-1137
#[derive(Debug, Clone, Default)]
pub struct RegisteredNodeCreateTransactionData {
    /// An administrative key controlled by the node operator.
    admin_key: Option<Key>,

    /// A short description of the node.
    description: String,

    /// A list of service endpoints for client calls.
    service_endpoints: Vec<RegisteredServiceEndpoint>,
}

impl RegisteredNodeCreateTransaction {
    /// Returns the admin key.
    #[must_use]
    pub fn get_admin_key(&self) -> Option<&Key> {
        self.data().admin_key.as_ref()
    }

    /// Sets the admin key.
    pub fn admin_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().admin_key = Some(key.into());
        self
    }

    /// Returns the description of the new node.
    #[must_use]
    pub fn get_description(&self) -> &str {
        &self.data().description
    }

    /// Sets the description of the new node.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.data_mut().description = description.into();
        self
    }

    /// Returns the list of service endpoints.
    #[must_use]
    pub fn get_service_endpoints(&self) -> Vec<RegisteredServiceEndpoint> {
        self.data().service_endpoints.clone()
    }

    /// Sets the list of service endpoints.
    pub fn service_endpoints(
        &mut self,
        service_endpoint: impl IntoIterator<Item = RegisteredServiceEndpoint>,
    ) -> &mut Self {
        self.data_mut().service_endpoints = service_endpoint.into_iter().collect();
        self
    }

    /// Adds a service endpoint to the list.
    pub fn add_service_endpoint(
        &mut self,
        service_endpoint: RegisteredServiceEndpoint,
    ) -> &mut Self {
        self.data_mut().service_endpoints.push(service_endpoint);
        self
    }
}

impl TransactionData for RegisteredNodeCreateTransactionData {}

impl TransactionExecute for RegisteredNodeCreateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async {
            AddressBookServiceClient::new(channel)
                .create_registered_node(request)
                .await
        })
    }
}

impl ValidateChecksums for RegisteredNodeCreateTransactionData {
    fn validate_checksums(&self, _ledger_id: &RefLedgerId) -> Result<(), Error> {
        Ok(())
    }
}

impl ToTransactionDataProtobuf for RegisteredNodeCreateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::RegisteredNodeCreate(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for RegisteredNodeCreateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::RegisteredNodeCreate(self.to_protobuf())
    }
}

impl From<RegisteredNodeCreateTransactionData> for AnyTransactionData {
    fn from(transaction: RegisteredNodeCreateTransactionData) -> Self {
        Self::RegisteredNodeCreate(transaction)
    }
}

impl FromProtobuf<services::RegisteredNodeCreateTransactionBody>
    for RegisteredNodeCreateTransactionData
{
    fn from_protobuf(pb: services::RegisteredNodeCreateTransactionBody) -> crate::Result<Self> {
        let service_endpoints = pb
            .service_endpoint
            .into_iter()
            .map(RegisteredServiceEndpoint::from_protobuf)
            .collect::<crate::Result<Vec<_>>>()?;

        Ok(Self {
            admin_key: Option::from_protobuf(pb.admin_key)?,
            description: pb.description,
            service_endpoints,
        })
    }
}

impl ToProtobuf for RegisteredNodeCreateTransactionData {
    type Protobuf = services::RegisteredNodeCreateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let service_endpoints =
            self.service_endpoints.iter().map(|it| it.to_protobuf()).collect::<Vec<_>>();

        services::RegisteredNodeCreateTransactionBody {
            admin_key: self.admin_key.to_protobuf(),
            description: self.description.clone(),
            service_endpoint: service_endpoints,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use hiero_sdk_proto::services;

    use super::RegisteredNodeCreateTransaction;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::registered_node::RegisteredNodeCreateTransactionData;
    use crate::registered_service_endpoint::{
        BlockNodeApi,
        IpAddress,
        RegisteredEndpointType,
        RegisteredServiceEndpoint,
    };
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
    };
    use crate::{
        AnyTransaction,
        Key,
    };

    const TEST_DESCRIPTION: &str = "test registered node";

    fn make_service_endpoints() -> Vec<RegisteredServiceEndpoint> {
        vec![RegisteredServiceEndpoint {
            ip_address: Some(IpAddress::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
            domain_name: String::new(),
            port: 50211,
            requires_tls: false,
            endpoint_type: Some(RegisteredEndpointType::BlockNode(BlockNodeApi::Status)),
        }]
    }

    fn make_transaction() -> RegisteredNodeCreateTransaction {
        let mut tx = RegisteredNodeCreateTransaction::new_for_tests();

        tx.admin_key(unused_private_key().public_key())
            .description(TEST_DESCRIPTION)
            .service_endpoints(make_service_endpoints())
            .freeze()
            .unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/registered_node_create_transaction/serialize.txt"]
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
        let tx = services::RegisteredNodeCreateTransactionBody {
            admin_key: Some(unused_private_key().public_key().to_protobuf()),
            description: TEST_DESCRIPTION.to_owned(),
            service_endpoint: make_service_endpoints()
                .into_iter()
                .map(|it| it.to_protobuf())
                .collect(),
        };

        let data = RegisteredNodeCreateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.admin_key, Some(Key::from(unused_private_key().public_key())));
        assert_eq!(data.description, TEST_DESCRIPTION);
        assert_eq!(data.service_endpoints.len(), 1);
    }

    #[test]
    fn get_set_admin_key() {
        let mut tx = RegisteredNodeCreateTransaction::new();
        tx.admin_key(unused_private_key().public_key());

        assert_eq!(tx.get_admin_key(), Some(&Key::from(unused_private_key().public_key())));
    }

    #[test]
    #[should_panic]
    fn get_set_admin_key_frozen_panic() {
        make_transaction().admin_key(Key::from(unused_private_key().public_key()));
    }

    #[test]
    fn get_set_description() {
        let mut tx = RegisteredNodeCreateTransaction::new();
        tx.description(TEST_DESCRIPTION);

        assert_eq!(tx.get_description(), TEST_DESCRIPTION);
    }

    #[test]
    #[should_panic]
    fn get_set_description_frozen_panic() {
        make_transaction().description(TEST_DESCRIPTION);
    }

    #[test]
    fn get_set_service_endpoints() {
        let endpoints = make_service_endpoints();
        let mut tx = RegisteredNodeCreateTransaction::new();
        tx.service_endpoints(endpoints.clone());

        assert_eq!(tx.get_service_endpoints(), endpoints);
    }

    #[test]
    #[should_panic]
    fn get_set_service_endpoints_frozen_panic() {
        make_transaction().service_endpoints(make_service_endpoints());
    }
}
