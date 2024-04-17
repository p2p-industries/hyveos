#![cfg_attr(not(feature = "std"), no_std)]

use core::{fmt::Display, net::Ipv6Addr, str::FromStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A EUI48 MAC address
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Eui48 {
    bytes: [u8; 6],
}

/// A EUI64 MAC address
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Eui64 {
    bytes: [u8; 8],
}

/// A MAC address
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MacAddress {
    Eui48(Eui48),
    Eui64(Eui64),
}

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum ParseMacError {
    #[cfg_attr(feature = "std", error("Invalid MAC address format"))]
    InvalidFormat,
    #[cfg_attr(feature = "std", error("Invalid hex value `{0}` in MAC address"))]
    InvalidHex(core::num::ParseIntError),
}

impl From<core::num::ParseIntError> for ParseMacError {
    fn from(err: core::num::ParseIntError) -> Self {
        ParseMacError::InvalidHex(err)
    }
}

impl Eui48 {
    /// Create a new EUI48 MAC address
    pub fn new(bytes: [u8; 6]) -> Self {
        Eui48 { bytes }
    }
}

impl Display for Eui48 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.bytes[0],
            self.bytes[1],
            self.bytes[2],
            self.bytes[3],
            self.bytes[4],
            self.bytes[5]
        )
    }
}

impl FromStr for Eui48 {
    type Err = ParseMacError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; 6];
        let mut i = 0;
        for part in s.split(':') {
            if i >= 6 {
                return Err(ParseMacError::InvalidFormat);
            }
            bytes[i] = u8::from_str_radix(part, 16)?;
            i += 1;
        }
        if i != 6 {
            return Err(ParseMacError::InvalidFormat);
        }
        Ok(Eui48 { bytes })
    }
}

impl FromStr for Eui64 {
    type Err = ParseMacError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; 8];
        let mut i = 0;
        for part in s.split(':') {
            if i >= 8 {
                return Err(ParseMacError::InvalidFormat);
            }
            bytes[i] = u8::from_str_radix(part, 16)?;
            i += 1;
        }
        if i != 8 {
            return Err(ParseMacError::InvalidFormat);
        }
        Ok(Eui64 { bytes })
    }
}

impl FromStr for MacAddress {
    type Err = ParseMacError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 17 {
            Ok(MacAddress::Eui48(Eui48::from_str(s)?))
        } else if s.len() == 23 {
            Ok(MacAddress::Eui64(Eui64::from_str(s)?))
        } else {
            Err(ParseMacError::InvalidFormat)
        }
    }
}

impl Eui64 {
    /// Create a new EUI64 MAC address
    pub fn new(bytes: [u8; 8]) -> Self {
        Eui64 { bytes }
    }
}

impl Display for Eui64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.bytes[0],
            self.bytes[1],
            self.bytes[2],
            self.bytes[3],
            self.bytes[4],
            self.bytes[5],
            self.bytes[6],
            self.bytes[7]
        )
    }
}

impl From<Eui48> for MacAddress {
    fn from(eui48: Eui48) -> Self {
        MacAddress::Eui48(eui48)
    }
}

impl From<Eui64> for MacAddress {
    fn from(eui64: Eui64) -> Self {
        MacAddress::Eui64(eui64)
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MacAddress::Eui48(eui48) => write!(f, "{}", eui48),
            MacAddress::Eui64(eui64) => write!(f, "{}", eui64),
        }
    }
}

impl From<[u8; 6]> for Eui48 {
    fn from(bytes: [u8; 6]) -> Self {
        Eui48 { bytes }
    }
}

impl From<[u8; 8]> for Eui64 {
    fn from(bytes: [u8; 8]) -> Self {
        Eui64 { bytes }
    }
}

impl From<[u8; 6]> for MacAddress {
    fn from(bytes: [u8; 6]) -> Self {
        MacAddress::Eui48(Eui48::from(bytes))
    }
}

impl From<[u8; 8]> for MacAddress {
    fn from(bytes: [u8; 8]) -> Self {
        MacAddress::Eui64(Eui64::from(bytes))
    }
}

impl From<Eui48> for Eui64 {
    fn from(eui48: Eui48) -> Self {
        let mut eui64 = [0u8; 8];
        eui64[0..3].copy_from_slice(&eui48.bytes[0..3]);

        eui64[3] = 0xFF;
        eui64[4] = 0xFE;

        eui64[5..8].copy_from_slice(&eui48.bytes[3..6]);

        eui64[0] ^= 0b0000_0010;

        eui64.into()
    }
}

impl From<Eui64> for Ipv6Addr {
    fn from(value: Eui64) -> Self {
        let mut prefix = [0u8; 16];
        prefix[0] = 0xFE;
        prefix[1] = 0x80;

        prefix[8..].copy_from_slice(&value.bytes);

        prefix.into()
    }
}

impl From<MacAddress> for Ipv6Addr {
    fn from(value: MacAddress) -> Self {
        match value {
            MacAddress::Eui48(mac) => Eui64::from(mac),
            MacAddress::Eui64(mac) => mac,
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::string::ToString;
    use core::net::Ipv6Addr;

    #[test]
    fn test_eui48_from_str() {
        use super::Eui48;

        let mac = Eui48::from_str("01:23:45:67:89:AB").unwrap();
        assert_eq!(mac.bytes, [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB]);
    }

    #[test]
    fn test_eui64_from_str() {
        use super::Eui64;

        let mac = Eui64::from_str("01:23:45:67:89:AB:CD:EF").unwrap();
        assert_eq!(mac.bytes, [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
    }

    #[test]
    fn test_eui48_display() {
        use super::Eui48;

        let mac = Eui48::from_str("01:23:45:67:89:AB").unwrap();
        assert_eq!(mac.to_string(), "01:23:45:67:89:AB");
    }

    #[test]
    fn test_eui64_display() {
        let mac = Eui64::from_str("01:23:45:67:89:AB:CD:EF").unwrap();
        assert_eq!(mac.to_string(), "01:23:45:67:89:AB:CD:EF");
    }

    #[test]
    fn test_eui48_to_eui64() {
        let mac = Eui48::from_str("01:23:45:67:89:AB").unwrap();
        let mac_64: Eui64 = mac.into();
        assert_eq!(
            mac_64.bytes,
            [0x03, 0x23, 0x45, 0xFF, 0xFE, 0x67, 0x89, 0xAB]
        );
    }

    #[test]
    fn test_eui64_to_ipv6() {
        let mac = Eui48::from_str("01:23:45:67:89:AB").unwrap();
        let mac_64: Eui64 = mac.into();
        let ip: Ipv6Addr = mac_64.into();
        assert_eq!(ip, Ipv6Addr::from_str("fe80::323:45ff:fe67:89ab").unwrap());
    }
}
