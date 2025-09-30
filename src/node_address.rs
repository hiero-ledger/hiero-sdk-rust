// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddrV4;

use crate::proto::services;

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    Error,
    FromProtobuf,
    ServiceEndpoint,
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

/// The data about a node, including its service endpoints and the Hiero account to be paid for
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

    /// A node's service endpoints as strings in format "host:port".
    pub service_endpoints: Vec<String>,

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
            let socket_addr = parse_socket_addr_v4(pb.ip_address, pb.portno)?;
            addresses.push(format!("{}:{}", socket_addr.ip(), socket_addr.port()));
        }

        for address in pb.service_endpoint {
            let endpoint = ServiceEndpoint::from_protobuf(address)?;
            let host = match endpoint.ip_address_v4 {
                Some(ip) => ip.to_string(),
                None => endpoint.domain_name,
            };
            addresses.push(format!("{}:{}", host, endpoint.port));
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
            .map(|endpoint_str| {
                // Parse "host:port" format back to ServiceEndpoint
                let parts: Vec<&str> = endpoint_str.split(':').collect();
                if parts.len() != 2 {
                    return services::ServiceEndpoint {
                        ip_address_v4: Vec::new(),
                        port: 50211,
                        domain_name: String::new(),
                    };
                }

                let host = parts[0];
                let port = parts[1].parse::<i32>().unwrap_or(50211);

                // Try to parse as IP address first, otherwise treat as domain name
                if let Ok(ip) = host.parse::<std::net::Ipv4Addr>() {
                    services::ServiceEndpoint {
                        ip_address_v4: ip.octets().to_vec(),
                        port,
                        domain_name: String::new(),
                    }
                } else {
                    services::ServiceEndpoint {
                        ip_address_v4: Vec::new(),
                        port,
                        domain_name: host.to_string(),
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_address_with_ip_endpoints() {
        let pb = services::NodeAddress {
            node_id: 3,
            rsa_pub_key: "746573745f6b6579".to_string(), // hex encoded "test_key"
            node_account_id: Some(AccountId::new(0, 0, 3).to_protobuf()),
            node_cert_hash: vec![1, 2, 3, 4],
            service_endpoint: vec![
                services::ServiceEndpoint {
                    ip_address_v4: vec![192, 168, 1, 1],
                    port: 50211,
                    domain_name: String::new(),
                },
                services::ServiceEndpoint {
                    ip_address_v4: vec![10, 0, 0, 1],
                    port: 50211,
                    domain_name: String::new(),
                },
            ],
            description: "Test node".to_string(),
            ..Default::default()
        };

        let node_address = NodeAddress::from_protobuf(pb).unwrap();
        assert_eq!(node_address.node_id, 3);
        assert_eq!(node_address.node_account_id, AccountId::new(0, 0, 3));
        assert_eq!(node_address.service_endpoints, vec!["192.168.1.1:50211", "10.0.0.1:50211"]);
        assert_eq!(node_address.description, "Test node");
    }

    #[test]
    fn test_node_address_with_domain_endpoints() {
        let pb = services::NodeAddress {
            node_id: 4,
            rsa_pub_key: "746573745f6b6579".to_string(), // hex encoded "test_key"
            node_account_id: Some(AccountId::new(0, 0, 4).to_protobuf()),
            node_cert_hash: vec![1, 2, 3, 4],
            service_endpoint: vec![
                services::ServiceEndpoint {
                    ip_address_v4: vec![],
                    port: 50211,
                    domain_name: "example.com".to_string(),
                },
                services::ServiceEndpoint {
                    ip_address_v4: vec![],
                    port: 50211,
                    domain_name: "localhost".to_string(),
                },
            ],
            description: "Test node with domains".to_string(),
            ..Default::default()
        };

        let node_address = NodeAddress::from_protobuf(pb).unwrap();
        assert_eq!(node_address.node_id, 4);
        assert_eq!(node_address.node_account_id, AccountId::new(0, 0, 4));
        assert_eq!(node_address.service_endpoints, vec!["example.com:50211", "localhost:50211"]);
        assert_eq!(node_address.description, "Test node with domains");
    }

    #[test]
    fn test_node_address_with_mixed_endpoints() {
        let pb = services::NodeAddress {
            node_id: 5,
            rsa_pub_key: "746573745f6b6579".to_string(), // hex encoded "test_key"
            node_account_id: Some(AccountId::new(0, 0, 5).to_protobuf()),
            node_cert_hash: vec![1, 2, 3, 4],
            service_endpoint: vec![
                services::ServiceEndpoint {
                    ip_address_v4: vec![192, 168, 1, 1],
                    port: 50211,
                    domain_name: String::new(),
                },
                services::ServiceEndpoint {
                    ip_address_v4: vec![],
                    port: 50211,
                    domain_name: "example.com".to_string(),
                },
            ],
            description: "Test node with mixed endpoints".to_string(),
            ..Default::default()
        };

        let node_address = NodeAddress::from_protobuf(pb).unwrap();
        assert_eq!(node_address.node_id, 5);
        assert_eq!(node_address.node_account_id, AccountId::new(0, 0, 5));
        assert_eq!(node_address.service_endpoints, vec!["192.168.1.1:50211", "example.com:50211"]);
        assert_eq!(node_address.description, "Test node with mixed endpoints");
    }

    #[test]
    fn test_node_address_to_protobuf() {
        let node_address = NodeAddress {
            node_id: 7,
            rsa_public_key: vec![1, 2, 3, 4],
            node_account_id: AccountId::new(0, 0, 7),
            tls_certificate_hash: vec![5, 6, 7, 8],
            service_endpoints: vec![
                "192.168.1.1:50211".to_string(),
                "example.com:50211".to_string(),
            ],
            description: "Test node for to_protobuf".to_string(),
        };

        let pb = node_address.to_protobuf();
        assert_eq!(pb.node_id, 7);
        assert_eq!(pb.rsa_pub_key, "01020304");
        assert_eq!(pb.node_cert_hash, vec![5, 6, 7, 8]);
        assert_eq!(pb.description, "Test node for to_protobuf");
        assert_eq!(pb.service_endpoint.len(), 2);

        // Check first endpoint (IP address)
        assert_eq!(pb.service_endpoint[0].ip_address_v4, vec![192, 168, 1, 1]);
        assert_eq!(pb.service_endpoint[0].port, 50211);
        assert_eq!(pb.service_endpoint[0].domain_name, "");

        // Check second endpoint (domain name)
        assert_eq!(pb.service_endpoint[1].ip_address_v4, vec![] as Vec<u8>);
        assert_eq!(pb.service_endpoint[1].port, 50211);
        assert_eq!(pb.service_endpoint[1].domain_name, "example.com");
    }

    #[test]
    fn test_node_address_round_trip() {
        let original = NodeAddress {
            node_id: 8,
            rsa_public_key: vec![1, 2, 3, 4],
            node_account_id: AccountId::new(0, 0, 8),
            tls_certificate_hash: vec![5, 6, 7, 8],
            service_endpoints: vec![
                "192.168.1.1:50211".to_string(),
                "example.com:50211".to_string(),
                "localhost:50211".to_string(),
            ],
            description: "Test node round trip".to_string(),
        };

        let pb = original.to_protobuf();
        let deserialized = NodeAddress::from_protobuf(pb).unwrap();

        assert_eq!(deserialized.node_id, original.node_id);
        assert_eq!(deserialized.node_account_id, original.node_account_id);
        assert_eq!(deserialized.service_endpoints, original.service_endpoints);
        assert_eq!(deserialized.description, original.description);
    }

    #[test]
    fn test_node_address_with_invalid_string_format() {
        let node_address = NodeAddress {
            node_id: 9,
            rsa_public_key: vec![1, 2, 3, 4],
            node_account_id: AccountId::new(0, 0, 9),
            tls_certificate_hash: vec![5, 6, 7, 8],
            service_endpoints: vec![
                "invalid-format".to_string(),           // Missing port
                "192.168.1.1:invalid-port".to_string(), // Invalid port
            ],
            description: "Test node with invalid strings".to_string(),
        };

        let pb = node_address.to_protobuf();
        // Should handle gracefully and use defaults
        assert_eq!(pb.service_endpoint.len(), 2);
        assert_eq!(pb.service_endpoint[0].port, 50211); // Default port
        assert_eq!(pb.service_endpoint[1].port, 50211); // Default port
    }

    #[test]
    fn test_node_address_with_localhost_and_127_0_0_1() {
        let pb = services::NodeAddress {
            node_id: 10,
            rsa_pub_key: "746573745f6b6579".to_string(), // hex encoded "test_key"
            node_account_id: Some(AccountId::new(0, 0, 10).to_protobuf()),
            node_cert_hash: vec![1, 2, 3, 4],
            service_endpoint: vec![
                services::ServiceEndpoint {
                    ip_address_v4: vec![],
                    port: 50211,
                    domain_name: "localhost".to_string(),
                },
                services::ServiceEndpoint {
                    ip_address_v4: vec![127, 0, 0, 1],
                    port: 50211,
                    domain_name: String::new(),
                },
            ],
            description: "Test node with localhost".to_string(),
            ..Default::default()
        };

        let node_address = NodeAddress::from_protobuf(pb).unwrap();
        assert_eq!(node_address.service_endpoints, vec!["localhost:50211", "127.0.0.1:50211"]);
    }

    #[test]
    fn test_node_address_with_kubernetes_style_domain() {
        let pb = services::NodeAddress {
            node_id: 11,
            rsa_pub_key: "746573745f6b6579".to_string(), // hex encoded "test_key"
            node_account_id: Some(AccountId::new(0, 0, 11).to_protobuf()),
            node_cert_hash: vec![1, 2, 3, 4],
            service_endpoint: vec![services::ServiceEndpoint {
                ip_address_v4: vec![],
                port: 50211,
                domain_name: "network-node1-svc.solo-e2e.svc.cluster.local".to_string(),
            }],
            description: "Test node with k8s domain".to_string(),
            ..Default::default()
        };

        let node_address = NodeAddress::from_protobuf(pb).unwrap();
        assert_eq!(
            node_address.service_endpoints,
            vec!["network-node1-svc.solo-e2e.svc.cluster.local:50211"]
        );
    }

    #[test]
    fn test_node_address_with_different_ports() {
        let pb = services::NodeAddress {
            node_id: 12,
            rsa_pub_key: "746573745f6b6579".to_string(), // hex encoded "test_key"
            node_account_id: Some(AccountId::new(0, 0, 12).to_protobuf()),
            node_cert_hash: vec![1, 2, 3, 4],
            service_endpoint: vec![
                services::ServiceEndpoint {
                    ip_address_v4: vec![192, 168, 1, 1],
                    port: 50211,
                    domain_name: String::new(),
                },
                services::ServiceEndpoint {
                    ip_address_v4: vec![10, 0, 0, 1],
                    port: 50212,
                    domain_name: String::new(),
                },
            ],
            description: "Test node with different ports".to_string(),
            ..Default::default()
        };

        let node_address = NodeAddress::from_protobuf(pb).unwrap();
        assert_eq!(node_address.service_endpoints, vec!["192.168.1.1:50211", "10.0.0.1:50212"]);
    }
}
