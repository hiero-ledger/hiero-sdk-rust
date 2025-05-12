// SPDX-License-Identifier: Apache-2.0

use std::net::Ipv4Addr;

use hedera_proto::services;
use hedera_proto::services::address_book_service_client::AddressBookServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::FromProtobuf;
use crate::service_endpoint::ServiceEndpoint;
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
    Error,
    Key,
    ToProtobuf,
    Transaction,
    ValidateChecksums,
};

/// A transaction body to add a new consensus node to the network address book.
///
/// This transaction body SHALL be considered a "privileged transaction".
///
/// This message supports a transaction to create a new node in the network
/// address book. The transaction, once complete, enables a new consensus node
/// to join the network, and requires governing council authorization.
pub type NodeCreateTransaction = Transaction<NodeCreateTransactionData>;

/// A transaction body to add a new consensus node to the network address book.
#[derive(Debug, Clone, Default)]
pub struct NodeCreateTransactionData {
    /// A Node account identifier.
    account_id: Option<AccountId>,

    /// A short description of the node.
    description: String,

    /// A list of service endpoints for gossip.
    gossip_endpoints: Vec<ServiceEndpoint>,

    /// A list of service endpoints for gRPC calls.
    service_endpoints: Vec<ServiceEndpoint>,

    /// A certificate used to sign gossip events.
    gossip_ca_certificate: Vec<u8>,

    /// A hash of the node gRPC TLS certificate.
    grpc_certificate_hash: Vec<u8>,

    /// An administrative key controlled by the node operator.
    admin_key: Option<Key>,
}

impl NodeCreateTransaction {
    /// Returns the account associated with the new node.
    #[must_use]
    pub fn get_account_id(&self) -> Option<AccountId> {
        self.data().account_id
    }

    /// Sets the account associated with the new node.
    pub fn account_id(&mut self, account_id: AccountId) -> &mut Self {
        self.data_mut().account_id = Some(account_id);
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

    /// Returns the list of service endpoints for gossip.
    #[must_use]
    pub fn get_gossip_endpoints(&self) -> Vec<ServiceEndpoint> {
        self.data().gossip_endpoints.clone()
    }

    /// Sets the list of service endpoints for gossip.
    pub fn gossip_endpoints(
        &mut self,
        gossip_endpoint: impl IntoIterator<Item = ServiceEndpoint>,
    ) -> &mut Self {
        self.data_mut().gossip_endpoints = gossip_endpoint.into_iter().collect();
        self
    }

    /// Adds a service endpoint for gossip to the list of service endpoints.
    pub fn add_gossip_endpoint(&mut self, gossip_endpoint: ServiceEndpoint) -> &mut Self {
        self.data_mut().gossip_endpoints.push(gossip_endpoint);
        self
    }

    /// Returns the list of service endpoints for gRPC calls.
    #[must_use]
    pub fn get_service_endpoints(&self) -> Vec<ServiceEndpoint> {
        self.data().service_endpoints.clone()
    }

    /// Sets the list of service endpoints for gRPC calls.
    pub fn service_endpoints(
        &mut self,
        service_endpoint: impl IntoIterator<Item = ServiceEndpoint>,
    ) -> &mut Self {
        self.data_mut().service_endpoints = service_endpoint.into_iter().collect();
        self
    }

    /// Adds a service endpoint to the list of service endpoints for gRPC calls.
    pub fn add_service_endpoint(&mut self, service_endpoint: ServiceEndpoint) -> &mut Self {
        self.data_mut().service_endpoints.push(service_endpoint);
        self
    }

    /// Returns the certificate used to sign gossip events.
    #[must_use]
    pub fn get_gossip_ca_certificate(&self) -> &[u8] {
        &self.data().gossip_ca_certificate
    }

    /// Sets the certificate used to sign gossip events.
    pub fn gossip_ca_certificate(
        &mut self,
        gossip_ca_certificate: impl Into<Vec<u8>>,
    ) -> &mut Self {
        self.data_mut().gossip_ca_certificate = gossip_ca_certificate.into();
        self
    }

    /// Returns the hash of the node gRPC TLS certificate.
    #[must_use]
    pub fn get_grpc_certificate_hash(&self) -> &[u8] {
        &self.data().grpc_certificate_hash
    }

    /// Sets the hash of the node gRPC TLS certificate.
    pub fn grpc_certificate_hash(
        &mut self,
        grpc_certificate_hash: impl Into<Vec<u8>>,
    ) -> &mut Self {
        self.data_mut().grpc_certificate_hash = grpc_certificate_hash.into();
        self
    }

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
}

impl TransactionData for NodeCreateTransactionData {}

impl TransactionExecute for NodeCreateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { AddressBookServiceClient::new(channel).create_node(request).await })
    }
}

impl ValidateChecksums for NodeCreateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.account_id.validate_checksums(ledger_id)?;
        Ok(())
    }
}

impl ToTransactionDataProtobuf for NodeCreateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::NodeCreate(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for NodeCreateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::NodeCreate(self.to_protobuf())
    }
}

impl From<NodeCreateTransactionData> for AnyTransactionData {
    fn from(transaction: NodeCreateTransactionData) -> Self {
        Self::NodeCreate(transaction)
    }
}

impl FromProtobuf<services::NodeCreateTransactionBody> for NodeCreateTransactionData {
    fn from_protobuf(pb: services::NodeCreateTransactionBody) -> crate::Result<Self> {
        let gossip_endpoints = pb
            .gossip_endpoint
            .iter()
            .map(|it| {
                let ip_addr_v4 = &it.ip_address_v4[..];
                let ip = Ipv4Addr::new(ip_addr_v4[0], ip_addr_v4[1], ip_addr_v4[2], ip_addr_v4[3]);
                ServiceEndpoint {
                    ip_address_v4: Some(ip),
                    port: it.port,
                    domain_name: it.domain_name.clone(),
                }
            })
            .collect();
        let service_endpoints = pb
            .service_endpoint
            .iter()
            .map(|it| {
                let ip_addr_v4 = &it.ip_address_v4[..];
                let ip = Ipv4Addr::new(ip_addr_v4[0], ip_addr_v4[1], ip_addr_v4[2], ip_addr_v4[3]);
                ServiceEndpoint {
                    ip_address_v4: Some(ip),
                    port: it.port,
                    domain_name: it.domain_name.clone(),
                }
            })
            .collect();

        Ok(Self {
            account_id: FromProtobuf::from_protobuf(pb.account_id)?,
            description: pb.description,
            gossip_endpoints: gossip_endpoints,
            service_endpoints: service_endpoints,
            gossip_ca_certificate: pb.gossip_ca_certificate,
            grpc_certificate_hash: pb.grpc_certificate_hash,
            admin_key: Option::from_protobuf(pb.admin_key)?,
        })
    }
}

impl ToProtobuf for NodeCreateTransactionData {
    type Protobuf = services::NodeCreateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let gossip_endpoints =
            self.gossip_endpoints.iter().map(|it| it.to_protobuf()).collect::<Vec<_>>();
        let service_endpoints =
            self.service_endpoints.iter().map(|it| it.to_protobuf()).collect::<Vec<_>>();

        services::NodeCreateTransactionBody {
            account_id: self.account_id.to_protobuf(),
            description: self.description.clone(),
            gossip_endpoint: gossip_endpoints,
            service_endpoint: service_endpoints,
            gossip_ca_certificate: self.gossip_ca_certificate.clone(),
            grpc_certificate_hash: self.grpc_certificate_hash.clone(),
            admin_key: self.admin_key.to_protobuf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use expect_test::expect_file;
    use hedera_proto::services;

    use super::NodeCreateTransaction;
    use crate::address_book::NodeCreateTransactionData;
    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::service_endpoint::ServiceEndpoint;
    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
        unused_private_key,
        TEST_ACCOUNT_ID,
    };
    use crate::{
        AnyTransaction,
        Key,
    };

    const TEST_DESCRIPTION: &str = "test description";
    const TEST_GOSSIP_CA_CERTIFICATE: &[u8] = &[1, 2, 3, 4];
    const TEST_GRPC_CERTIFICATE_HASH: &[u8] = &[5, 6, 7, 8];

    fn make_ip_address_list() -> Vec<ServiceEndpoint> {
        vec![ServiceEndpoint {
            ip_address_v4: Some(Ipv4Addr::new(127, 0, 0, 1)),
            port: 1234,
            domain_name: "".to_owned(),
        }]
    }

    fn make_transaction() -> NodeCreateTransaction {
        let mut tx = NodeCreateTransaction::new_for_tests();

        tx.account_id(TEST_ACCOUNT_ID)
            .description(TEST_DESCRIPTION)
            .gossip_endpoints(make_ip_address_list())
            .service_endpoints(make_ip_address_list())
            .gossip_ca_certificate(TEST_GOSSIP_CA_CERTIFICATE)
            .grpc_certificate_hash(TEST_GRPC_CERTIFICATE_HASH)
            .admin_key(unused_private_key().public_key())
            .freeze()
            .unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect_file!["./snapshots/node_create_transaction/serialize.txt"].assert_debug_eq(&tx);
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
        let tx = services::NodeCreateTransactionBody {
            account_id: Some(TEST_ACCOUNT_ID.to_protobuf()),
            description: TEST_DESCRIPTION.to_owned(),
            gossip_endpoint: make_ip_address_list()
                .into_iter()
                .map(|it| it.to_protobuf())
                .collect(),
            service_endpoint: make_ip_address_list()
                .into_iter()
                .map(|it| it.to_protobuf())
                .collect(),
            gossip_ca_certificate: TEST_GOSSIP_CA_CERTIFICATE.to_vec(),
            grpc_certificate_hash: TEST_GRPC_CERTIFICATE_HASH.to_vec(),
            admin_key: Some(unused_private_key().public_key().to_protobuf()),
        };

        let data = NodeCreateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.account_id, Some(TEST_ACCOUNT_ID));
        assert_eq!(data.description, TEST_DESCRIPTION);
        assert_eq!(data.gossip_endpoints, make_ip_address_list());
        assert_eq!(data.service_endpoints, make_ip_address_list());
        assert_eq!(data.gossip_ca_certificate, TEST_GOSSIP_CA_CERTIFICATE);
        assert_eq!(data.grpc_certificate_hash, TEST_GRPC_CERTIFICATE_HASH);
        assert_eq!(data.admin_key, Some(Key::from(unused_private_key().public_key())));
    }

    #[test]
    fn get_set_account_id() {
        let account_id = TEST_ACCOUNT_ID;
        let mut tx = NodeCreateTransaction::new();
        tx.account_id(account_id.to_owned());

        assert_eq!(tx.get_account_id(), Some(account_id));
    }

    #[test]
    #[should_panic]
    fn get_set_account_id_frozen_panic() {
        make_transaction().account_id(TEST_ACCOUNT_ID);
    }

    #[test]
    fn get_set_description() {
        let description = TEST_DESCRIPTION.to_owned();
        let mut tx = NodeCreateTransaction::new();
        tx.description(description.to_owned());

        assert_eq!(tx.get_description(), TEST_DESCRIPTION);
    }

    #[test]
    #[should_panic]
    fn get_set_description_frozen_panic() {
        make_transaction().description(TEST_DESCRIPTION);
    }

    #[test]
    fn get_set_gossip_endpoints() {
        let gossip_endpoints = make_ip_address_list();
        let mut tx = NodeCreateTransaction::new();
        tx.gossip_endpoints(gossip_endpoints.to_owned());

        assert_eq!(tx.get_gossip_endpoints(), gossip_endpoints);
    }

    #[test]
    #[should_panic]
    fn get_set_gossip_endpoint_frozen_panic() {
        make_transaction().gossip_endpoints(make_ip_address_list());
    }

    #[test]
    fn get_set_service_endpoints() {
        let service_endpoints = make_ip_address_list();
        let mut tx = NodeCreateTransaction::new();
        tx.service_endpoints(service_endpoints.to_owned());

        assert_eq!(tx.get_service_endpoints(), service_endpoints);
    }

    #[test]
    #[should_panic]
    fn get_set_service_endpoints_frozen_panic() {
        make_transaction().service_endpoints(make_ip_address_list());
    }

    #[test]
    fn get_set_grpc_certificate_hash() {
        let mut tx = NodeCreateTransaction::new();
        tx.grpc_certificate_hash(TEST_GOSSIP_CA_CERTIFICATE);

        assert_eq!(tx.get_grpc_certificate_hash(), TEST_GOSSIP_CA_CERTIFICATE);
    }

    #[test]
    #[should_panic]
    fn get_set_grpc_certificate_hash_frozen_panic() {
        make_transaction().grpc_certificate_hash(TEST_GOSSIP_CA_CERTIFICATE);
    }

    #[test]
    fn get_set_admin_key() {
        let mut tx = NodeCreateTransaction::new();
        tx.admin_key(unused_private_key().public_key());

        assert_eq!(tx.get_admin_key(), Some(&Key::from(unused_private_key().public_key())));
    }

    #[test]
    #[should_panic]
    fn get_set_admin_key_frozen_panic() {
        make_transaction().admin_key(Key::from(unused_private_key().public_key()));
    }
}
