// SPDX-License-Identifier: Apache-2.0

use std::net::{
    Ipv4Addr,
    Ipv6Addr,
};

use hiero_sdk_proto::services;
use hiero_sdk_proto::services::registered_service_endpoint;

use crate::protobuf::ToProtobuf;
use crate::FromProtobuf;

/// An enumeration of well-known block node endpoint APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockNodeApi {
    /// Any other API type associated with a block node.
    Other,
    /// The Block Node Status API.
    Status,
    /// The Block Node Publish API.
    Publish,
    /// The Block Node Subscribe Stream API.
    SubscribeStream,
    /// The Block Node State Proof API.
    StateProof,
}

impl BlockNodeApi {
    fn from_proto(value: i32) -> crate::Result<Self> {
        match value {
            0 => Ok(Self::Other),
            1 => Ok(Self::Status),
            2 => Ok(Self::Publish),
            3 => Ok(Self::SubscribeStream),
            4 => Ok(Self::StateProof),
            _ => Err(crate::Error::from_protobuf(format!(
                "unknown BlockNodeApi value: {value}"
            ))),
        }
    }

    fn to_proto(self) -> i32 {
        match self {
            Self::Other => 0,
            Self::Status => 1,
            Self::Publish => 2,
            Self::SubscribeStream => 3,
            Self::StateProof => 4,
        }
    }
}

/// The type of a registered service endpoint.
#[derive(Debug, Clone, PartialEq)]
pub enum RegisteredEndpointType {
    /// A Block Node endpoint with supported APIs.
    BlockNode(Vec<BlockNodeApi>),
    /// A Mirror Node endpoint.
    MirrorNode,
    /// An RPC Relay endpoint.
    RpcRelay,
    /// A general service endpoint with an optional description.
    GeneralService(String),
}

/// The address of a registered service endpoint.
#[derive(Debug, Clone, PartialEq)]
pub enum RegisteredEndpointAddress {
    /// An IPv4 address.
    Ipv4(Ipv4Addr),
    /// An IPv6 address.
    Ipv6(Ipv6Addr),
    /// A fully qualified domain name.
    DomainName(String),
}

/// A registered network node endpoint.
///
/// Each registered network node in the global address book publishes one or
/// more endpoints which enable the nodes to communicate both with other
/// nodes and with clients to receive requests.
#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredServiceEndpoint {
    /// The address of this endpoint (IP or FQDN).
    pub address: RegisteredEndpointAddress,

    /// The network port (0-65535).
    pub port: u32,

    /// Whether TLS is required for this endpoint.
    pub requires_tls: bool,

    /// The type of service at this endpoint.
    pub endpoint_type: RegisteredEndpointType,
}

impl FromProtobuf<services::RegisteredServiceEndpoint> for RegisteredServiceEndpoint {
    fn from_protobuf(pb: services::RegisteredServiceEndpoint) -> crate::Result<Self> {
        let address = match pb.address {
            Some(registered_service_endpoint::Address::IpAddress(bytes)) => {
                if bytes.len() == 4 {
                    Ok(RegisteredEndpointAddress::Ipv4(Ipv4Addr::new(
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    )))
                } else if bytes.len() == 16 {
                    let octets: [u8; 16] = bytes.try_into().unwrap();
                    Ok(RegisteredEndpointAddress::Ipv6(Ipv6Addr::from(octets)))
                } else {
                    Err(crate::Error::from_protobuf(format!(
                        "expected 4 or 16 byte IP address, got {} bytes",
                        bytes.len()
                    )))
                }
            }
            Some(registered_service_endpoint::Address::DomainName(name)) => {
                Ok(RegisteredEndpointAddress::DomainName(name))
            }
            None => {
                Err(crate::Error::from_protobuf("RegisteredServiceEndpoint missing address"))
            }
        }?;

        let endpoint_type = match pb.endpoint_type {
            Some(registered_service_endpoint::EndpointType::BlockNode(bn)) => {
                let apis = bn
                    .endpoint_api
                    .into_iter()
                    .map(BlockNodeApi::from_proto)
                    .collect::<crate::Result<Vec<_>>>()?;
                Ok(RegisteredEndpointType::BlockNode(apis))
            }
            Some(registered_service_endpoint::EndpointType::MirrorNode(_)) => {
                Ok(RegisteredEndpointType::MirrorNode)
            }
            Some(registered_service_endpoint::EndpointType::RpcRelay(_)) => {
                Ok(RegisteredEndpointType::RpcRelay)
            }
            Some(registered_service_endpoint::EndpointType::GeneralService(gs)) => {
                Ok(RegisteredEndpointType::GeneralService(gs.description))
            }
            None => Err(crate::Error::from_protobuf(
                "RegisteredServiceEndpoint missing endpoint_type",
            )),
        }?;

        Ok(Self { address, port: pb.port, requires_tls: pb.requires_tls, endpoint_type })
    }
}

impl ToProtobuf for RegisteredServiceEndpoint {
    type Protobuf = services::RegisteredServiceEndpoint;

    fn to_protobuf(&self) -> Self::Protobuf {
        let address = Some(match &self.address {
            RegisteredEndpointAddress::Ipv4(ip) => {
                registered_service_endpoint::Address::IpAddress(ip.octets().to_vec())
            }
            RegisteredEndpointAddress::Ipv6(ip) => {
                registered_service_endpoint::Address::IpAddress(ip.octets().to_vec())
            }
            RegisteredEndpointAddress::DomainName(name) => {
                registered_service_endpoint::Address::DomainName(name.clone())
            }
        });

        let endpoint_type = Some(match &self.endpoint_type {
            RegisteredEndpointType::BlockNode(apis) => {
                registered_service_endpoint::EndpointType::BlockNode(
                    registered_service_endpoint::BlockNodeEndpoint {
                        endpoint_api: apis.iter().map(|a| a.to_proto()).collect(),
                    },
                )
            }
            RegisteredEndpointType::MirrorNode => {
                registered_service_endpoint::EndpointType::MirrorNode(
                    registered_service_endpoint::MirrorNodeEndpoint {},
                )
            }
            RegisteredEndpointType::RpcRelay => {
                registered_service_endpoint::EndpointType::RpcRelay(
                    registered_service_endpoint::RpcRelayEndpoint {},
                )
            }
            RegisteredEndpointType::GeneralService(desc) => {
                registered_service_endpoint::EndpointType::GeneralService(
                    registered_service_endpoint::GeneralServiceEndpoint {
                        description: desc.clone(),
                    },
                )
            }
        });

        services::RegisteredServiceEndpoint {
            port: self.port,
            requires_tls: self.requires_tls,
            address,
            endpoint_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{
        Ipv4Addr,
        Ipv6Addr,
    };

    use super::*;

    #[test]
    fn block_node_endpoint_round_trip() {
        let endpoint = RegisteredServiceEndpoint {
            address: RegisteredEndpointAddress::Ipv4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 8080,
            requires_tls: true,
            endpoint_type: RegisteredEndpointType::BlockNode(vec![
                BlockNodeApi::Status,
                BlockNodeApi::Publish,
            ]),
        };

        let pb = endpoint.to_protobuf();
        let deserialized = RegisteredServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized, endpoint);
    }

    #[test]
    fn mirror_node_endpoint_with_domain() {
        let endpoint = RegisteredServiceEndpoint {
            address: RegisteredEndpointAddress::DomainName("mirror.example.com".to_string()),
            port: 443,
            requires_tls: true,
            endpoint_type: RegisteredEndpointType::MirrorNode,
        };

        let pb = endpoint.to_protobuf();
        let deserialized = RegisteredServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized, endpoint);
    }

    #[test]
    fn rpc_relay_endpoint() {
        let endpoint = RegisteredServiceEndpoint {
            address: RegisteredEndpointAddress::Ipv4(Ipv4Addr::new(10, 0, 0, 1)),
            port: 7546,
            requires_tls: false,
            endpoint_type: RegisteredEndpointType::RpcRelay,
        };

        let pb = endpoint.to_protobuf();
        let deserialized = RegisteredServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized, endpoint);
    }

    #[test]
    fn general_service_endpoint() {
        let endpoint = RegisteredServiceEndpoint {
            address: RegisteredEndpointAddress::DomainName("service.example.com".to_string()),
            port: 9090,
            requires_tls: false,
            endpoint_type: RegisteredEndpointType::GeneralService("My custom service".to_string()),
        };

        let pb = endpoint.to_protobuf();
        let deserialized = RegisteredServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized, endpoint);
    }

    #[test]
    fn ipv6_address() {
        let endpoint = RegisteredServiceEndpoint {
            address: RegisteredEndpointAddress::Ipv6(Ipv6Addr::LOCALHOST),
            port: 8080,
            requires_tls: false,
            endpoint_type: RegisteredEndpointType::BlockNode(vec![BlockNodeApi::SubscribeStream]),
        };

        let pb = endpoint.to_protobuf();
        let deserialized = RegisteredServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized, endpoint);
    }
}
