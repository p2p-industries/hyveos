use std::{
    borrow::Cow,
    fmt::Display,
    io,
    net::{Ipv6Addr, SocketAddrV6},
    str::FromStr,
};

use libp2p::{multiaddr::Protocol, Multiaddr};
use macaddress::Eui64;

pub fn if_name_to_index(name: impl Into<Vec<u8>>) -> io::Result<u32> {
    let ifname = std::ffi::CString::new(name)?;
    match unsafe { libc::if_nametoindex(ifname.as_ptr()) } {
        0 => Err(io::Error::last_os_error()),
        otherwise => Ok(otherwise),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Empty string")]
    EmptyString,
    #[error("Missing scope ID")]
    MissingScopeId,
    #[error("Invalid ip address: {0}")]
    InvalidIp(#[from] std::net::AddrParseError),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Invalid multiaddr")]
    InvalidMultiaddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfAddr {
    pub if_index: u32,
    pub addr: Ipv6Addr,
}

impl IfAddr {
    pub fn with_port(&self, port: u16) -> std::net::SocketAddr {
        SocketAddrV6::new(self.addr, port, 0, self.if_index).into()
    }
}

impl FromStr for IfAddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '%');
        let addr = parts.next().ok_or(Error::EmptyString)?;
        let scope_id = parts.next().ok_or(Error::MissingScopeId)?;

        let addr = Ipv6Addr::from_str(addr).map_err(Error::from)?;
        let if_index = scope_id
            .parse()
            .or_else(|_| if_name_to_index(scope_id))
            .map_err(Error::from)?;

        Ok(Self { if_index, addr })
    }
}

impl Display for IfAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%{}", self.addr, self.if_index)
    }
}

impl From<IfAddr> for Multiaddr {
    fn from(addr: IfAddr) -> Self {
        let mac: Eui64 = addr.addr.into();

        let mut bytes = [0; 10];
        bytes[0..8].copy_from_slice(&mac.bytes());

        let mut multiaddr = Multiaddr::empty();
        multiaddr.push(Protocol::Onion(Cow::Owned(bytes), addr.if_index as u16));
        multiaddr
    }
}

impl TryFrom<&Multiaddr> for IfAddr {
    type Error = Error;

    fn try_from(multiaddr: &Multiaddr) -> Result<Self, Self::Error> {
        multiaddr
            .iter()
            .find_map(|segment| {
                if let Protocol::Onion(bytes, if_index) = segment {
                    let mut mac_bytes = [0; 8];
                    mac_bytes.copy_from_slice(&bytes[..8]);

                    let mac = Eui64::new(mac_bytes);
                    let addr = Ipv6Addr::from(mac);

                    Some(IfAddr {
                        if_index: if_index as u32,
                        addr,
                    })
                } else {
                    None
                }
            })
            .ok_or(Error::InvalidMultiaddr)
    }
}

impl TryFrom<Multiaddr> for IfAddr {
    type Error = Error;

    fn try_from(multiaddr: Multiaddr) -> Result<Self, Self::Error> {
        TryFrom::try_from(&multiaddr)
    }
}
