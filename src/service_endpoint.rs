// SPDX-License-Identifier: Apache-2.0

use std::net::{
    Ipv4Addr,
    SocketAddrV4,
};

use hedera_proto::services;

use crate::protobuf::ToProtobuf;
use crate::{
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

fn validate_domain_name(domain_name: String) -> crate::Result<()> {
    if domain_name.len() > 253 {
        return Err(Error::from_protobuf("Domain name exceeds 253 characters"));
    }

    // Check for valid characters and format
    if !domain_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.') {
        return Err(Error::from_protobuf("Invalid characters in domain name"));
    }

    // Check for valid domain name format (simplified)
    if !domain_name.contains('.') || domain_name.starts_with('.') || domain_name.ends_with('.') {
        return Err(Error::from_protobuf("Invalid domain name format"));
    }

    Ok(())
}

/// Contains the IP address, the port, and the domain name representing a service endpoint of
/// a Node in a network. Used to reach the Hedera API and submit transactions
/// to the network.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceEndpoint {
    /// The 4-byte IPv4 address of the endpoint encoded in left to right order
    pub ip_address_v4: Option<Ipv4Addr>,

    /// The port of the service endpoint
    pub port: i32,

    /// A node domain name.<br/>
    /// This MUST be the fully qualified domain(DNS) name of the node.<br/>
    /// This value MUST NOT be more than 253 characters.
    /// domain_name and ipAddressV4 are mutually exclusive.
    /// When the `domain_name` field is set, the `ipAddressV4` field MUST NOT be set.<br/>
    /// When the `ipAddressV4` field is set, the `domain_name` field MUST NOT be set.
    pub domain_name: String,
}

impl FromProtobuf<services::ServiceEndpoint> for ServiceEndpoint {
    fn from_protobuf(pb: services::ServiceEndpoint) -> crate::Result<Self> {
        let mut port = pb.port;
        if pb.port == 0 || pb.port == 50111 {
            port = 50211;
        }

        let socket_addr_v4 = parse_socket_addr_v4(pb.ip_address_v4, port)?;

        if !pb.domain_name.is_empty() {
            validate_domain_name(pb.domain_name.clone())?;
        }

        Ok(Self {
            ip_address_v4: Some(socket_addr_v4.ip().to_owned()),
            port: socket_addr_v4.port() as i32,
            domain_name: pb.domain_name,
        })
    }
}

impl ToProtobuf for ServiceEndpoint {
    type Protobuf = services::ServiceEndpoint;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::ServiceEndpoint {
            ip_address_v4: self.ip_address_v4.unwrap().octets().to_vec(),
            port: self.port,
            domain_name: self.domain_name.clone(),
        }
    }
}
