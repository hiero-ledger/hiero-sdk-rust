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

/// Transaction body to modify address book node attributes.
///
/// - This transaction SHALL enable the node operator, as identified by the
///    `admin_key`, to modify operational attributes of the node.
/// - This transaction MUST be signed by the active `admin_key` for the node.
/// - If this transaction sets a new value for the `admin_key`, then both the
///    current `admin_key`, and the new `admin_key` MUST sign this transaction.
/// - This transaction SHALL NOT change any field that is not set (is null) in
///    this transaction body.
/// - This SHALL create a pending update to the node, but the change SHALL NOT
///    be immediately applied to the active configuration.
/// - All pending node updates SHALL be applied to the active network
///    configuration during the next `freeze` transaction with the field
///    `freeze_type` set to `PREPARE_UPGRADE`.
///
/// ### Record Stream Effects
/// Upon completion the `node_id` for the updated entry SHALL be in the
/// transaction receipt.
pub type NodeUpdateTransaction = Transaction<NodeUpdateTransactionData>;

/// Transaction body to modify address book node attributes.
#[derive(Debug, Clone, Default)]
pub struct NodeUpdateTransactionData {
    /// A consensus node identifier in the network state.
    node_id: u64,

    /// A Node account identifier.
    account_id: Option<AccountId>,

    /// A short description of the node.
    description: Option<String>,

    /// A list of service endpoints for gossip.
    gossip_endpoints: Vec<ServiceEndpoint>,

    /// A list of service endpoints for gRPC calls.
    service_endpoints: Vec<ServiceEndpoint>,

    /// A certificate used to sign gossip events.
    gossip_ca_certificate: Option<Vec<u8>>,

    /// A hash of the node gRPC TLS certificate.
    grpc_certificate_hash: Option<Vec<u8>>,

    /// An administrative key controlled by the node operator.
    admin_key: Option<Key>,

    /// A flag indicating whether the node operator declines rewards.
    decline_reward: Option<bool>,

    /// A service endpoint for gRPC proxy.
    grpc_proxy_endpoint: Option<ServiceEndpoint>,
}

impl NodeUpdateTransaction {
    /// Returns the account associated with the new node.
    #[must_use]
    pub fn get_node_id(&self) -> u64 {
        self.data().node_id
    }

    /// Sets the account associated with the new node.
    pub fn node_id(&mut self, node_id: u64) -> &mut Self {
        self.data_mut().node_id = node_id;
        self
    }

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
    pub fn get_description(&self) -> Option<&str> {
        self.data().description.as_deref()
    }

    /// Sets the description of the new node.
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.data_mut().description = Some(description.into());
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

    /// Returns the updated list of service endpoints for gRPC calls.
    #[must_use]
    pub fn get_service_endpoints(&self) -> Vec<ServiceEndpoint> {
        self.data().service_endpoints.clone()
    }

    /// Sets the updated list of service endpoints for gRPC calls.
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

    /// Returns the updated certificate used to sign gossip events.
    #[must_use]
    pub fn get_gossip_ca_certificate(&self) -> Option<Vec<u8>> {
        self.data().gossip_ca_certificate.clone()
    }

    /// Updates the certificate used to sign gossip events.
    pub fn gossip_ca_certificate(
        &mut self,
        gossip_ca_certificate: impl Into<Vec<u8>>,
    ) -> &mut Self {
        self.data_mut().gossip_ca_certificate = Some(gossip_ca_certificate.into());
        self
    }

    /// Returns the updated hash of the node gRPC TLS certificate.
    #[must_use]
    pub fn get_grpc_certificate_hash(&self) -> Option<Vec<u8>> {
        self.data().grpc_certificate_hash.clone()
    }

    /// Updates the hash of the node gRPC TLS certificate.
    pub fn grpc_certificate_hash(
        &mut self,
        grpc_certificate_hash: impl Into<Vec<u8>>,
    ) -> &mut Self {
        self.data_mut().grpc_certificate_hash = Some(grpc_certificate_hash.into());
        self
    }

    /// Returns the updated admin key.
    #[must_use]
    pub fn get_admin_key(&self) -> Option<&Key> {
        self.data().admin_key.as_ref()
    }

    /// Updated the admin key.
    pub fn admin_key(&mut self, key: impl Into<Key>) -> &mut Self {
        self.data_mut().admin_key = Some(key.into());
        self
    }

    /// Returns the decline reward.
    #[must_use]
    pub fn get_decline_reward(&self) -> Option<bool> {
        self.data().decline_reward
    }

    /// Sets the decline reward.
    pub fn decline_reward(&mut self, decline_reward: bool) -> &mut Self {
        self.data_mut().decline_reward = Some(decline_reward);
        self
    }

    /// Returns the grpc proxy endpoint.
    #[must_use]
    pub fn get_grpc_proxy_endpoint(&self) -> Option<&ServiceEndpoint> {
        self.data().grpc_proxy_endpoint.as_ref()
    }

    /// Sets the grpc proxy endpoint.
    pub fn grpc_proxy_endpoint(&mut self, grpc_proxy_endpoint: ServiceEndpoint) -> &mut Self {
        self.data_mut().grpc_proxy_endpoint = Some(grpc_proxy_endpoint);
        self
    }
}

impl TransactionData for NodeUpdateTransactionData {}

impl TransactionExecute for NodeUpdateTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { AddressBookServiceClient::new(channel).update_node(request).await })
    }
}

impl ValidateChecksums for NodeUpdateTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.account_id.validate_checksums(ledger_id)?;
        Ok(())
    }
}

impl ToTransactionDataProtobuf for NodeUpdateTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::NodeUpdate(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for NodeUpdateTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::NodeUpdate(self.to_protobuf())
    }
}

impl From<NodeUpdateTransactionData> for AnyTransactionData {
    fn from(transaction: NodeUpdateTransactionData) -> Self {
        Self::NodeUpdate(transaction)
    }
}

impl FromProtobuf<services::NodeUpdateTransactionBody> for NodeUpdateTransactionData {
    fn from_protobuf(pb: services::NodeUpdateTransactionBody) -> crate::Result<Self> {
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
            node_id: pb.node_id,
            account_id: FromProtobuf::from_protobuf(pb.account_id)?,
            description: pb.description,
            gossip_endpoints: gossip_endpoints,
            service_endpoints: service_endpoints,
            gossip_ca_certificate: pb.gossip_ca_certificate,
            grpc_certificate_hash: pb.grpc_certificate_hash,
            admin_key: Option::from_protobuf(pb.admin_key)?,
            decline_reward: pb.decline_reward,
            grpc_proxy_endpoint: pb.grpc_proxy_endpoint.map(|it| ServiceEndpoint {
                ip_address_v4: Some(Ipv4Addr::new(
                    it.ip_address_v4[0],
                    it.ip_address_v4[1],
                    it.ip_address_v4[2],
                    it.ip_address_v4[3],
                )),
                port: it.port,
                domain_name: it.domain_name.clone(),
            }),
        })
    }
}

impl ToProtobuf for NodeUpdateTransactionData {
    type Protobuf = services::NodeUpdateTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let gossip_endpoints =
            self.gossip_endpoints.iter().map(|it| it.to_protobuf()).collect::<Vec<_>>();
        let service_endpoints =
            self.service_endpoints.iter().map(|it| it.to_protobuf()).collect::<Vec<_>>();

        services::NodeUpdateTransactionBody {
            node_id: self.node_id,
            account_id: self.account_id.to_protobuf(),
            description: self.description.clone(),
            gossip_endpoint: gossip_endpoints,
            service_endpoint: service_endpoints,
            gossip_ca_certificate: self.gossip_ca_certificate.clone(),
            grpc_certificate_hash: self.grpc_certificate_hash.clone(),
            admin_key: self.admin_key.to_protobuf(),
            decline_reward: self.decline_reward,
            grpc_proxy_endpoint: self.grpc_proxy_endpoint.as_ref().map(|it| it.to_protobuf()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use expect_test::expect_file;
    use hedera_proto::services;

    use super::NodeUpdateTransaction;
    use crate::address_book::NodeUpdateTransactionData;
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
        vec![
            ServiceEndpoint {
                ip_address_v4: Some(Ipv4Addr::new(127, 0, 0, 1)),
                port: 1234,
                domain_name: "".to_owned(),
            },
            ServiceEndpoint {
                ip_address_v4: Some(Ipv4Addr::new(127, 0, 0, 1)),
                port: 8008,
                domain_name: "".to_owned(),
            },
        ]
    }

    fn make_transaction() -> NodeUpdateTransaction {
        let mut tx = NodeUpdateTransaction::new_for_tests();

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

        expect_file!["./snapshots/node_update_transaction/serialize.txt"].assert_debug_eq(&tx);
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
        let grpc_proxy_endpoint = make_ip_address_list().into_iter().next().unwrap();
        let tx = services::NodeUpdateTransactionBody {
            node_id: 1,
            account_id: Some(TEST_ACCOUNT_ID.to_protobuf()),
            description: Some(TEST_DESCRIPTION.to_owned()),
            gossip_endpoint: make_ip_address_list()
                .into_iter()
                .map(|it| it.to_protobuf())
                .collect(),
            service_endpoint: make_ip_address_list()
                .into_iter()
                .map(|it| it.to_protobuf())
                .collect(),
            gossip_ca_certificate: Some(TEST_GOSSIP_CA_CERTIFICATE.to_vec()),
            grpc_certificate_hash: Some(TEST_GRPC_CERTIFICATE_HASH.to_vec()),
            admin_key: Some(unused_private_key().public_key().to_protobuf()),
            decline_reward: Some(false),
            grpc_proxy_endpoint: Some(grpc_proxy_endpoint.to_protobuf()),
        };

        let data = NodeUpdateTransactionData::from_protobuf(tx).unwrap();

        assert_eq!(data.account_id, Some(TEST_ACCOUNT_ID));
        assert_eq!(data.description, Some(TEST_DESCRIPTION.to_string()));
        assert_eq!(data.gossip_endpoints, make_ip_address_list());
        assert_eq!(data.service_endpoints, make_ip_address_list());
        assert_eq!(data.gossip_ca_certificate, Some(TEST_GOSSIP_CA_CERTIFICATE.to_vec()));
        assert_eq!(data.grpc_certificate_hash, Some(TEST_GRPC_CERTIFICATE_HASH.to_vec()));
        assert_eq!(data.admin_key, Some(Key::from(unused_private_key().public_key())));
        assert_eq!(data.decline_reward, Some(false));
        assert_eq!(data.grpc_proxy_endpoint, Some(grpc_proxy_endpoint));
    }

    #[test]
    fn get_set_node_id() {
        let mut tx = NodeUpdateTransaction::new();
        tx.node_id(1);

        assert_eq!(tx.get_node_id(), 1);
    }

    #[test]
    #[should_panic]
    fn get_set_node_id_frozen_panic() {
        make_transaction().node_id(1);
    }

    #[test]
    fn get_set_account_id() {
        let account_id = TEST_ACCOUNT_ID;
        let mut tx = NodeUpdateTransaction::new();
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
        let mut tx = NodeUpdateTransaction::new();
        tx.description(description.to_owned());

        assert_eq!(tx.get_description(), Some(TEST_DESCRIPTION));
    }

    #[test]
    #[should_panic]
    fn get_set_description_frozen_panic() {
        make_transaction().description(TEST_DESCRIPTION);
    }

    #[test]
    fn get_set_gossip_endpoints() {
        let gossip_endpoints = make_ip_address_list();
        let mut tx = NodeUpdateTransaction::new();
        tx.gossip_endpoints(gossip_endpoints.to_owned());

        assert_eq!(tx.get_gossip_endpoints(), gossip_endpoints);
    }

    #[test]
    #[should_panic]
    fn get_set_gossip_endpoint_frozen_panic() {
        make_transaction().gossip_endpoints(make_ip_address_list());
    }

    #[test]
    fn get_set_decline_reward() {
        let mut tx = NodeUpdateTransaction::new();
        tx.decline_reward(true);

        assert_eq!(tx.get_decline_reward(), Some(true));
    }

    #[test]
    fn get_set_grpc_proxy_endpoint() {
        let grpc_proxy_endpoint = make_ip_address_list().into_iter().next().unwrap();
        let mut tx = NodeUpdateTransaction::new();
        tx.grpc_proxy_endpoint(grpc_proxy_endpoint.clone());

        assert_eq!(tx.get_grpc_proxy_endpoint(), Some(&grpc_proxy_endpoint));
    }

    #[test]
    fn get_set_service_endpoints() {
        let service_endpoints = make_ip_address_list();
        let mut tx = NodeUpdateTransaction::new();
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
        let mut tx = NodeUpdateTransaction::new();
        tx.grpc_certificate_hash(TEST_GOSSIP_CA_CERTIFICATE);

        assert_eq!(tx.get_grpc_certificate_hash(), Some(TEST_GOSSIP_CA_CERTIFICATE.to_vec()));
    }

    #[test]
    #[should_panic]
    fn get_set_grpc_certificate_hash_frozen_panic() {
        make_transaction().grpc_certificate_hash(TEST_GOSSIP_CA_CERTIFICATE);
    }

    #[test]
    fn get_set_admin_key() {
        let mut tx = NodeUpdateTransaction::new();
        tx.admin_key(unused_private_key().public_key());

        assert_eq!(tx.get_admin_key(), Some(&Key::from(unused_private_key().public_key())));
    }

    #[test]
    #[should_panic]
    fn get_set_admin_key_frozen_panic() {
        make_transaction().admin_key(Key::from(unused_private_key().public_key()));
    }
}
