use std::{
    borrow::Cow,
    fmt::Display,
    io,
    net::{Ipv6Addr, SocketAddr, SocketAddrV6},
    str::FromStr,
    sync::Arc,
};

use multiaddr::{Multiaddr, Protocol};

pub fn if_name_to_index(name: impl Into<Vec<u8>>) -> io::Result<u32> {
    let ifname = std::ffi::CString::new(name)?;
    match unsafe { libc::if_nametoindex(ifname.as_ptr()) } {
        0 => Err(io::Error::last_os_error()),
        otherwise => Ok(otherwise),
    }
}

pub fn if_index_to_name(name: u32) -> io::Result<String> {
    let mut buffer = [0; libc::IF_NAMESIZE];
    let maybe_name = unsafe { libc::if_indextoname(name, buffer.as_mut_ptr()) };
    if maybe_name.is_null() {
        Err(io::Error::last_os_error())
    } else {
        let cstr = unsafe { std::ffi::CStr::from_ptr(maybe_name) };
        Ok(cstr.to_string_lossy().into_owned())
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfAddr {
    pub if_index: u32,
    pub if_name: Arc<str>,
    pub addr: Ipv6Addr,
}

impl IfAddr {
    pub fn new_with_index(addr: Ipv6Addr, if_index: u32) -> io::Result<Self> {
        Ok(Self {
            if_index,
            if_name: if_index_to_name(if_index)?.into(),
            addr,
        })
    }

    pub fn new_with_name(addr: Ipv6Addr, if_name: impl AsRef<str>) -> io::Result<Self> {
        let if_name = if_name.as_ref();

        Ok(Self {
            if_index: if_name_to_index(if_name)?,
            if_name: if_name.into(),
            addr,
        })
    }

    pub fn with_port(&self, port: u16) -> io::Result<SocketAddr> {
        Ok(SocketAddrV6::new(self.addr, port, 0, self.if_index).into())
    }

    pub fn to_multiaddr(&self, name: bool) -> Multiaddr {
        let zone_str = if name {
            Cow::Borrowed(self.if_name.as_ref())
        } else {
            Cow::from(self.if_index.to_string())
        };

        Multiaddr::empty()
            .with(Protocol::Ip6(self.addr))
            .with(Protocol::Ip6zone(zone_str))
    }
}

impl Display for IfAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%{}", self.addr, self.if_name)
    }
}

impl FromStr for IfAddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let mut parts = s.splitn(2, '%');
        let addr = parts.next().ok_or(Error::EmptyString)?;
        let zone_str = parts.next().ok_or(Error::MissingScopeId)?;

        let addr = Ipv6Addr::from_str(addr).map_err(Error::from)?;

        if let Ok(index) = zone_str.parse() {
            Self::new_with_index(addr, index)
        } else {
            Self::new_with_name(addr, zone_str)
        }
        .map_err(Into::into)
    }
}

impl TryFrom<&Multiaddr> for IfAddr {
    type Error = Error;

    fn try_from(multiaddr: &Multiaddr) -> Result<Self, Self::Error> {
        let mut iter = multiaddr.iter();

        let Some(Protocol::Ip6(addr)) = iter.next() else {
            return Err(Error::InvalidMultiaddr);
        };

        let Some(Protocol::Ip6zone(zone_str)) = iter.next() else {
            return Err(Error::InvalidMultiaddr);
        };

        if let Ok(index) = zone_str.parse() {
            Self::new_with_index(addr, index)
        } else {
            Self::new_with_name(addr, zone_str)
        }
        .map_err(Into::into)
    }
}

impl TryFrom<Multiaddr> for IfAddr {
    type Error = Error;

    fn try_from(multiaddr: Multiaddr) -> Result<Self, Self::Error> {
        IfAddr::try_from(&multiaddr)
    }
}
