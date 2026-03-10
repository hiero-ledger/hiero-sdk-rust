// SPDX-License-Identifier: Apache-2.0

use std::net::{
    Ipv4Addr,
    SocketAddrV4,
};

use hiero_sdk_proto::services;

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
    // Allow localhost and single-word domains for local development
    if domain_name == "localhost" {
        return Ok(());
    }

    if !domain_name.contains('.') || domain_name.starts_with('.') || domain_name.ends_with('.') {
        return Err(Error::from_protobuf("Invalid domain name format"));
    }

    Ok(())
}

/// Contains the IP address, the port, and the domain name representing a service endpoint of
/// a Node in a network. Used to reach the Hiero API and submit transactions
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

        // Only parse IP address if it's present
        let ip_address_v4 = if !pb.ip_address_v4.is_empty() {
            let socket_addr_v4 = parse_socket_addr_v4(pb.ip_address_v4, port)?;
            Some(socket_addr_v4.ip().to_owned())
        } else {
            None
        };

        if !pb.domain_name.is_empty() {
            validate_domain_name(pb.domain_name.clone())?;
        }

        Ok(Self { ip_address_v4, port, domain_name: pb.domain_name })
    }
}

impl ToProtobuf for ServiceEndpoint {
    type Protobuf = services::ServiceEndpoint;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::ServiceEndpoint {
            ip_address_v4: self.ip_address_v4.map(|ip| ip.octets().to_vec()).unwrap_or_default(),
            port: self.port,
            domain_name: self.domain_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn test_service_endpoint_with_ip_address() {
        let ip = Ipv4Addr::new(192, 168, 1, 1);
        let endpoint =
            ServiceEndpoint { ip_address_v4: Some(ip), port: 50211, domain_name: String::new() };

        let pb = endpoint.to_protobuf();
        assert_eq!(pb.ip_address_v4, vec![192, 168, 1, 1]);
        assert_eq!(pb.port, 50211);
        assert_eq!(pb.domain_name, "");

        let deserialized = ServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized.ip_address_v4, Some(ip));
        assert_eq!(deserialized.port, 50211);
        assert_eq!(deserialized.domain_name, "");
    }

    #[test]
    fn test_service_endpoint_with_domain_name() {
        let endpoint = ServiceEndpoint {
            ip_address_v4: None,
            port: 50211,
            domain_name: "example.com".to_string(),
        };

        let pb = endpoint.to_protobuf();
        assert_eq!(pb.ip_address_v4, vec![] as Vec<u8>);
        assert_eq!(pb.port, 50211);
        assert_eq!(pb.domain_name, "example.com");

        let deserialized = ServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized.ip_address_v4, None);
        assert_eq!(deserialized.port, 50211);
        assert_eq!(deserialized.domain_name, "example.com");
    }

    #[test]
    fn test_service_endpoint_with_empty_ip_address() {
        let endpoint = ServiceEndpoint {
            ip_address_v4: None,
            port: 50211,
            domain_name: "localhost".to_string(),
        };

        let pb = endpoint.to_protobuf();
        assert_eq!(pb.ip_address_v4, vec![] as Vec<u8>);
        assert_eq!(pb.port, 50211);
        assert_eq!(pb.domain_name, "localhost");

        let deserialized = ServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized.ip_address_v4, None);
        assert_eq!(deserialized.port, 50211);
        assert_eq!(deserialized.domain_name, "localhost");
    }

    #[test]
    fn test_service_endpoint_port_defaulting() {
        // Test port 0 gets defaulted to 50211
        let pb = services::ServiceEndpoint {
            ip_address_v4: vec![192, 168, 1, 1],
            port: 0,
            domain_name: String::new(),
        };

        let endpoint = ServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(endpoint.port, 50211);

        // Test port 50111 gets defaulted to 50211
        let pb = services::ServiceEndpoint {
            ip_address_v4: vec![192, 168, 1, 1],
            port: 50111,
            domain_name: String::new(),
        };

        let endpoint = ServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(endpoint.port, 50211);
    }

    #[test]
    fn test_service_endpoint_domain_name_validation() {
        // Valid domain name
        let pb = services::ServiceEndpoint {
            ip_address_v4: vec![],
            port: 50211,
            domain_name: "valid-domain.com".to_string(),
        };

        let result = ServiceEndpoint::from_protobuf(pb);
        assert!(result.is_ok());

        // Invalid domain name (too long)
        let long_domain = "a".repeat(254);
        let pb = services::ServiceEndpoint {
            ip_address_v4: vec![],
            port: 50211,
            domain_name: long_domain,
        };

        let result = ServiceEndpoint::from_protobuf(pb);
        assert!(result.is_err());

        // Invalid domain name (invalid characters)
        let pb = services::ServiceEndpoint {
            ip_address_v4: vec![],
            port: 50211,
            domain_name: "invalid@domain.com".to_string(),
        };

        let result = ServiceEndpoint::from_protobuf(pb);
        assert!(result.is_err());

        // Invalid domain name (starts with dot)
        let pb = services::ServiceEndpoint {
            ip_address_v4: vec![],
            port: 50211,
            domain_name: ".domain.com".to_string(),
        };

        let result = ServiceEndpoint::from_protobuf(pb);
        assert!(result.is_err());

        // Invalid domain name (ends with dot)
        let pb = services::ServiceEndpoint {
            ip_address_v4: vec![],
            port: 50211,
            domain_name: "domain.com.".to_string(),
        };

        let result = ServiceEndpoint::from_protobuf(pb);
        assert!(result.is_err());

        // Invalid domain name (no dots)
        let pb = services::ServiceEndpoint {
            ip_address_v4: vec![],
            port: 50211,
            domain_name: "domain".to_string(),
        };

        let result = ServiceEndpoint::from_protobuf(pb);
        assert!(result.is_err());
    }

    #[test]
    fn test_service_endpoint_round_trip() {
        let original = ServiceEndpoint {
            ip_address_v4: Some(Ipv4Addr::new(10, 0, 0, 1)),
            port: 50211,
            domain_name: "test.example.com".to_string(),
        };

        let pb = original.to_protobuf();
        let deserialized = ServiceEndpoint::from_protobuf(pb).unwrap();

        assert_eq!(deserialized.ip_address_v4, original.ip_address_v4);
        assert_eq!(deserialized.port, original.port);
        assert_eq!(deserialized.domain_name, original.domain_name);
    }

    #[test]
    fn test_service_endpoint_with_localhost() {
        let endpoint = ServiceEndpoint {
            ip_address_v4: None,
            port: 50211,
            domain_name: "localhost".to_string(),
        };

        let pb = endpoint.to_protobuf();
        assert_eq!(pb.ip_address_v4, vec![] as Vec<u8>);
        assert_eq!(pb.port, 50211);
        assert_eq!(pb.domain_name, "localhost");

        let deserialized = ServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized.ip_address_v4, None);
        assert_eq!(deserialized.port, 50211);
        assert_eq!(deserialized.domain_name, "localhost");
    }

    #[test]
    fn test_service_endpoint_with_127_0_0_1() {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let endpoint =
            ServiceEndpoint { ip_address_v4: Some(ip), port: 50211, domain_name: String::new() };

        let pb = endpoint.to_protobuf();
        assert_eq!(pb.ip_address_v4, vec![127, 0, 0, 1]);
        assert_eq!(pb.port, 50211);
        assert_eq!(pb.domain_name, "");

        let deserialized = ServiceEndpoint::from_protobuf(pb).unwrap();
        assert_eq!(deserialized.ip_address_v4, Some(ip));
        assert_eq!(deserialized.port, 50211);
        assert_eq!(deserialized.domain_name, "");
    }
}
