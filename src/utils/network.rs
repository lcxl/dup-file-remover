use std::{net::{IpAddr, Ipv6Addr}, str::FromStr};

pub fn check_ipv6_available() -> bool {

    let addr = "::1"; // Loopback IPv6 address

    match IpAddr::from_str(addr) {

        Ok(IpAddr::V6(ipv6_addr)) => true,

        _ => false,

    }

}