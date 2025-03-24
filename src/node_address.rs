// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddrV4;

use hedera_proto::services;

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    Error,
    FromProtobuf,
};

fn parse_socket_addr_v4(ip: Vec<u8>, port: i32) -> crate::Result<SocketAddrV4> {
    let octets: Result<[u8; 4], _> = ip.try_into();
    let octets = octets.map_err(|v| {
        Error::from_protobuf(format!("expected 4 byte ip address, got `{}` bytes", v.len()))
    })?;

    let port = u16::try_from(port).map_err(|_| {
        Error::from_protobuf(format!(
            "expected 16 bit non-negative port number, but the port was actually `{port}`",
        ))
    })?;

    Ok(SocketAddrV4::new(octets.into(), port))
}

/// The data about a node, including its service endpoints and the Hedera account to be paid for
/// services provided by the node (that is, queries answered and transactions submitted.).
#[derive(Debug, Clone)]
pub struct NodeAddress {
    /// A non-sequential, unique, static identifier for the node
    pub node_id: u64,

    /// The node's X509 RSA public key used to sign stream files.
    pub rsa_public_key: Vec<u8>,

    /// The account to be paid for queries and transactions sent to this node.
    pub node_account_id: AccountId,

    /// Hash of the node's TLS certificate.
    ///
    /// Precisely, this field is a string of
    /// hexadecimal characters which, translated to binary, are the SHA-384 hash of
    /// the UTF-8 NFKD encoding of the node's TLS cert in PEM format.
    ///
    /// Its value can be used to verify the node's certificate it presents during TLS negotiations.
    pub tls_certificate_hash: Vec<u8>,

    /// A node's service IP addresses and ports.
    pub service_endpoints: Vec<SocketAddrV4>,

    /// A description of the node, up to 100 bytes.
    pub description: String,
}

impl FromProtobuf<services::NodeAddress> for NodeAddress {
    fn from_protobuf(pb: services::NodeAddress) -> crate::Result<Self>
    where
        Self: Sized,
    {
        // sometimes this will be oversized by 1, but that's fine.
        let mut addresses = Vec::with_capacity(pb.service_endpoint.len() + 1);

        // `ip_address`/`portno` are deprecated, but lets handle them anyway.
        #[allow(deprecated)]
        if !pb.ip_address.is_empty() {
            addresses.push(parse_socket_addr_v4(pb.ip_address, pb.portno)?);
        }

        for address in pb.service_endpoint {
            addresses.push(parse_socket_addr_v4(address.ip_address_v4, address.port)?);
        }

        let node_account_id = AccountId::from_protobuf(pb_getf!(pb, node_account_id)?)?;

        Ok(Self {
            description: pb.description,
            rsa_public_key: hex::decode(pb.rsa_pub_key).map_err(Error::from_protobuf)?,
            node_id: pb.node_id as u64,
            service_endpoints: addresses,
            tls_certificate_hash: pb.node_cert_hash,
            node_account_id,
        })
    }
}

impl ToProtobuf for NodeAddress {
    type Protobuf = services::NodeAddress;

    fn to_protobuf(&self) -> Self::Protobuf {
        let service_endpoint = self
            .service_endpoints
            .iter()
            .map(|it| services::ServiceEndpoint {
                ip_address_v4: it.ip().octets().to_vec(),
                port: i32::from(it.port()),
                domain_name: it.to_string(),
            })
            .collect();

        services::NodeAddress {
            rsa_pub_key: hex::encode(&self.rsa_public_key),
            node_id: self.node_id as i64,
            node_account_id: Some(self.node_account_id.to_protobuf()),
            node_cert_hash: self.tls_certificate_hash.clone(),
            service_endpoint,
            description: self.description.clone(),

            // deprecated fields
            ..Default::default()
        }
    }
}
