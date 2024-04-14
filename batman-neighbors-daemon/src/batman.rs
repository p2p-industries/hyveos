use std::io;

use batman_neighbors_core::BatmanNeighbor;
use byteorder::{ByteOrder as _, NativeEndian};
use netlink_packet_generic::{GenlFamily, GenlHeader};
use netlink_packet_utils::{
    nla::{Nla, NlasIterator},
    parsers, DecodeError, Emitable, ParseableParametrized,
};

const BATADV_ATTR_MESH_IFINDEX: u16 = 3;
const BATADV_ATTR_HARD_IFINDEX: u16 = 6;
const BATADV_ATTR_HARD_IFNAME: u16 = 7;
const BATADV_ATTR_LAST_SEEN_MSECS: u16 = 23;
const BATADV_ATTR_NEIGH_ADDRESS: u16 = 24;
const BATADV_ATTR_THROUGHPUT: u16 = 26;

const BATADV_CMD_GET_NEIGHBORS: u8 = 9;

fn if_name_to_index(name: impl Into<Vec<u8>>) -> io::Result<u32> {
    let ifname = std::ffi::CString::new(name)?;
    #[allow(unsafe_code)]
    match unsafe { libc::if_nametoindex(ifname.as_ptr()) } {
        0 => Err(std::io::Error::last_os_error()),
        otherwise => Ok(otherwise),
    }
}

fn if_index_to_name(index: u32) -> io::Result<String> {
    let ifname = unsafe { libc::if_indextoname(index, std::ptr::null_mut()) };
    if ifname.is_null() {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { std::ffi::CStr::from_ptr(ifname) }
            .to_string_lossy()
            .into_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MeshIfIndex(u32);

impl Nla for MeshIfIndex {
    fn value_len(&self) -> usize {
        std::mem::size_of::<u32>()
    }

    fn kind(&self) -> u16 {
        BATADV_ATTR_MESH_IFINDEX
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        NativeEndian::write_u32(buffer, self.0);
    }
}

impl TryFrom<&str> for MeshIfIndex {
    type Error = io::Error;

    fn try_from(value: &str) -> io::Result<Self> {
        let index = if_name_to_index(value)?;
        Ok(Self(index))
    }
}

#[derive(Debug, Clone)]
pub enum MessageRequestCommand {
    GetNeighbors,
}

impl MessageRequestCommand {
    fn get_cmd(&self) -> u8 {
        match self {
            Self::GetNeighbors => BATADV_CMD_GET_NEIGHBORS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageRequest {
    cmd: MessageRequestCommand,
    if_index: MeshIfIndex,
}

impl Emitable for MessageRequest {
    fn buffer_len(&self) -> usize {
        self.if_index.buffer_len() + match self.cmd {
            _ => 0
        }
    }

    fn emit(&self, buffer: &mut [u8]) {
        match self.cmd {
            _ => {}
        }

        self.if_index.emit(buffer);
    }
}

#[derive(Debug, Clone)]
pub enum MessageResponseCommand {
    Neighbor(BatmanNeighbor),
}

impl MessageResponseCommand {
    fn parse_neighbor(buffer: &[u8]) -> Result<Self, DecodeError> {
        let mut if_index = None;
        let mut if_name = None;
        let mut last_seen_msecs = None;
        let mut mac = None;
        let mut throughput_kbps = None;

        for nla in NlasIterator::new(buffer) {
            let nla = &nla.map_err(|e| format!("Received invalid data from kernel: {}", e))?;
            match nla.kind() {
                BATADV_ATTR_HARD_IFINDEX => {
                    if_index = Some(parsers::parse_u32(nla.value())?)
                }
                BATADV_ATTR_HARD_IFNAME => {
                    if_name = Some(parsers::parse_string(nla.value())?)
                }
                BATADV_ATTR_LAST_SEEN_MSECS => {
                    last_seen_msecs = Some(parsers::parse_u32(nla.value())?)
                }
                BATADV_ATTR_NEIGH_ADDRESS => {
                    mac = Some(parsers::parse_mac(nla.value())?)
                }
                BATADV_ATTR_THROUGHPUT => {
                    throughput_kbps = Some(parsers::parse_u32(nla.value())?)
                }
                _ => {}
            }
        }

        let if_name = if let Some(if_name) = if_name {
            if_name
        } else if let Some(if_index) = if_index {
            if_index_to_name(if_index).unwrap_or("".into())
        } else {
            return Err("Missing attribute if_name or if_index from kernel".into())
        };

        let last_seen_msecs = last_seen_msecs.ok_or("Missing attribute last_seen_msecs from kernel")?;
        let mac = mac.ok_or("Missing attribute mac from kernel")?;

        Ok(Self::Neighbor(BatmanNeighbor {
            if_name,
            last_seen_msecs,
            mac,
            throughput_kbps,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct MessageResponse {
    pub cmd: MessageResponseCommand,
}

impl ParseableParametrized<[u8], GenlHeader> for MessageResponse {
    fn parse_with_param(
        buffer: &[u8],
        header: GenlHeader,
    ) -> Result<Self, DecodeError> {
        Ok(Self {
            cmd: match header.cmd {
                BATADV_CMD_GET_NEIGHBORS => MessageResponseCommand::parse_neighbor(buffer)?,
                cmd => {
                    return Err(DecodeError::from(format!(
                        "Unsupported batadv response command: {}",
                        cmd
                    )))
                }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Request(MessageRequest),
    Response(MessageResponse),
}

impl Message {
    pub fn new_request(cmd: MessageRequestCommand, if_name: &str) -> io::Result<Self> {
        Ok(Self::Request(MessageRequest {
            cmd,
            if_index: if_name.try_into()?,
        }))
    }
}

impl GenlFamily for Message {
    fn family_name() -> &'static str {
        "batadv"
    }

    fn command(&self) -> u8 {
        if let Self::Request(req) = self {
            req.cmd.get_cmd()
        } else {
            panic!("Unexpected response")
        }
    }

    fn version(&self) -> u8 {
        1
    }
}

impl Emitable for Message {
    fn buffer_len(&self) -> usize {
        if let Self::Request(req) = self {
            req.buffer_len()
        } else {
            panic!("Unexpected response")
        }
    }

    fn emit(&self, buffer: &mut [u8]) {
        if let Self::Request(req) = self {
            req.emit(buffer)
        } else {
            panic!("Unexpected response")
        }
    }
}

impl ParseableParametrized<[u8], GenlHeader> for Message {
    fn parse_with_param(
        buffer: &[u8],
        header: GenlHeader,
    ) -> Result<Self, DecodeError> {
        MessageResponse::parse_with_param(buffer, header).map(Self::Response)
    }
}
