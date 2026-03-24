// SPDX-License-Identifier: Apache-2.0

use std::net::{
    Ipv4Addr,
    Ipv6Addr,
};

use hiero_sdk_proto::services;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};

/// The type of endpoint for a registered node.
///
/// HIP-1137
#[derive(Debug, Clone, PartialEq)]
pub enum RegisteredEndpointType {
    /// A Block Node endpoint.
    BlockNode(BlockNodeApi),

    /// A Mirror Node endpoint.
    MirrorNode,

    /// A RPC Relay endpoint.
    RpcRelay,

    /// A general service endpoint.
    GeneralService(String),
}

/// The API type for a Block Node endpoint.
///
/// HIP-1137
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockNodeApi {
    /// Any other API type.
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

/// A registered network node endpoint.
///
/// Each registered network node publishes one or more endpoints which enable
/// other nodes and clients to communicate with it.
///
/// HIP-1137
#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredServiceEndpoint {
    /// An IP address (IPv4 or IPv6) for the endpoint, or `None` if using a domain name.
    pub ip_address: Option<IpAddress>,

    /// A fully qualified domain name for the endpoint, or empty if using an IP address.
    pub domain_name: String,

    /// The port number (0-65535).
    pub port: u32,

    /// Whether TLS is required for this endpoint.
    pub requires_tls: bool,

    /// The type of endpoint.
    pub endpoint_type: Option<RegisteredEndpointType>,
}

/// An IP address, either IPv4 or IPv6.
#[derive(Debug, Clone, PartialEq)]
pub enum IpAddress {
    /// An IPv4 address.
    V4(Ipv4Addr),
    /// An IPv6 address.
    V6(Ipv6Addr),
}

impl FromProtobuf<services::RegisteredServiceEndpoint> for RegisteredServiceEndpoint {
    fn from_protobuf(pb: services::RegisteredServiceEndpoint) -> crate::Result<Self> {
        use services::registered_service_endpoint::Address;
        use services::registered_service_endpoint::EndpointType;

        let (ip_address, domain_name) = match pb.address {
            Some(Address::IpAddress(bytes)) => {
                let ip = match bytes.len() {
                    4 => IpAddress::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])),
                    16 => {
                        let mut octets = [0u8; 16];
                        octets.copy_from_slice(&bytes);
                        IpAddress::V6(Ipv6Addr::from(octets))
                    }
                    _ => {
                        return Err(crate::Error::from_protobuf(
                            "invalid IP address length in RegisteredServiceEndpoint",
                        ));
                    }
                };
                (Some(ip), String::new())
            }
            Some(Address::DomainName(name)) => (None, name),
            None => (None, String::new()),
        };

        let endpoint_type = match pb.endpoint_type {
            Some(EndpointType::BlockNode(bn)) => {
                use services::registered_service_endpoint::block_node_endpoint::BlockNodeApi as PbApi;
                let api = match PbApi::try_from(bn.endpoint_api) {
                    Ok(PbApi::Other) => BlockNodeApi::Other,
                    Ok(PbApi::Status) => BlockNodeApi::Status,
                    Ok(PbApi::Publish) => BlockNodeApi::Publish,
                    Ok(PbApi::SubscribeStream) => BlockNodeApi::SubscribeStream,
                    Ok(PbApi::StateProof) => BlockNodeApi::StateProof,
                    _ => BlockNodeApi::Other,
                };
                Some(RegisteredEndpointType::BlockNode(api))
            }
            Some(EndpointType::MirrorNode(_)) => Some(RegisteredEndpointType::MirrorNode),
            Some(EndpointType::RpcRelay(_)) => Some(RegisteredEndpointType::RpcRelay),
            Some(EndpointType::GeneralService(gs)) => {
                Some(RegisteredEndpointType::GeneralService(gs.description))
            }
            None => None,
        };

        Ok(Self { ip_address, domain_name, port: pb.port, requires_tls: pb.requires_tls, endpoint_type })
    }
}

impl ToProtobuf for RegisteredServiceEndpoint {
    type Protobuf = services::RegisteredServiceEndpoint;

    fn to_protobuf(&self) -> Self::Protobuf {
        use services::registered_service_endpoint::block_node_endpoint::BlockNodeApi as PbApi;
        use services::registered_service_endpoint::{
            Address,
            BlockNodeEndpoint,
            EndpointType,
            GeneralServiceEndpoint,
            MirrorNodeEndpoint,
            RpcRelayEndpoint,
        };

        let address = if let Some(ip) = &self.ip_address {
            match ip {
                IpAddress::V4(v4) => Some(Address::IpAddress(v4.octets().to_vec())),
                IpAddress::V6(v6) => Some(Address::IpAddress(v6.octets().to_vec())),
            }
        } else if !self.domain_name.is_empty() {
            Some(Address::DomainName(self.domain_name.clone()))
        } else {
            None
        };

        let endpoint_type = self.endpoint_type.as_ref().map(|et| match et {
            RegisteredEndpointType::BlockNode(api) => {
                let pb_api = match api {
                    BlockNodeApi::Other => PbApi::Other,
                    BlockNodeApi::Status => PbApi::Status,
                    BlockNodeApi::Publish => PbApi::Publish,
                    BlockNodeApi::SubscribeStream => PbApi::SubscribeStream,
                    BlockNodeApi::StateProof => PbApi::StateProof,
                };
                EndpointType::BlockNode(BlockNodeEndpoint { endpoint_api: pb_api as i32 })
            }
            RegisteredEndpointType::MirrorNode => {
                EndpointType::MirrorNode(MirrorNodeEndpoint {})
            }
            RegisteredEndpointType::RpcRelay => EndpointType::RpcRelay(RpcRelayEndpoint {}),
            RegisteredEndpointType::GeneralService(desc) => {
                EndpointType::GeneralService(GeneralServiceEndpoint {
                    description: desc.clone(),
                })
            }
        });

        services::RegisteredServiceEndpoint {
            address,
            port: self.port,
            requires_tls: self.requires_tls,
            endpoint_type,
        }
    }
}
