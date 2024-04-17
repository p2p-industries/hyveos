use std::{io, time::Duration};

use batman_neighbours_core::BatmanNeighbour;
use byteorder::{ByteOrder as _, NativeEndian};
use netlink_packet_generic::{GenlFamily, GenlHeader};
use netlink_packet_utils::{
    nla::{Nla, NlasIterator},
    parsers, DecodeError, Emitable, ParseableParametrized,
};

const BATADV_ATTR_MESH_IFINDEX: u16 = 3;
const BATADV_ATTR_HARD_IFINDEX: u16 = 6;
const BATADV_ATTR_LAST_SEEN_MSECS: u16 = 23;
const BATADV_ATTR_NEIGH_ADDRESS: u16 = 24;
const BATADV_ATTR_THROUGHPUT: u16 = 26;

const BATADV_CMD_GET_NEIGHBOURS: u8 = 9;

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

#[derive(Debug, Clone)]
pub enum MessageRequestCommand {
    GetNeighbours,
}

impl MessageRequestCommand {
    fn get_cmd(&self) -> u8 {
        match self {
            Self::GetNeighbours => BATADV_CMD_GET_NEIGHBOURS,
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
        self.if_index.buffer_len()
            + match self.cmd {
                _ => 0,
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
    Neighbour(BatmanNeighbour),
}

impl MessageResponseCommand {
    fn parse_neighbour(buffer: &[u8]) -> Result<Self, DecodeError> {
        let mut if_index = None;
        let mut last_seen_msecs = None;
        let mut mac = None;
        let mut throughput_kbps = None;

        for nla in NlasIterator::new(buffer) {
            let nla = &nla.map_err(|e| format!("Received invalid data from kernel: {}", e))?;
            match nla.kind() {
                BATADV_ATTR_HARD_IFINDEX => if_index = Some(parsers::parse_u32(nla.value())?),
                BATADV_ATTR_LAST_SEEN_MSECS => {
                    last_seen_msecs = Some(parsers::parse_u32(nla.value())?)
                }
                BATADV_ATTR_NEIGH_ADDRESS => mac = Some(parsers::parse_mac(nla.value())?),
                BATADV_ATTR_THROUGHPUT => throughput_kbps = Some(parsers::parse_u32(nla.value())?),
                _ => {}
            }
        }

        let if_index = if_index.ok_or("Missing attribute if_name or if_index from kernel")?;
        let last_seen_msecs =
            last_seen_msecs.ok_or("Missing attribute last_seen_msecs from kernel")?;
        let mac = mac.ok_or("Missing attribute mac from kernel")?;

        Ok(Self::Neighbour(BatmanNeighbour {
            if_index,
            last_seen: Duration::from_millis(last_seen_msecs as u64),
            mac: mac.into(),
            throughput_kbps,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct MessageResponse {
    pub cmd: MessageResponseCommand,
}

impl ParseableParametrized<[u8], GenlHeader> for MessageResponse {
    fn parse_with_param(buffer: &[u8], header: GenlHeader) -> Result<Self, DecodeError> {
        Ok(Self {
            cmd: match header.cmd {
                BATADV_CMD_GET_NEIGHBOURS => MessageResponseCommand::parse_neighbour(buffer)?,
                cmd => {
                    return Err(DecodeError::from(format!(
                        "Unsupported batadv response command: {}",
                        cmd
                    )))
                }
            },
        })
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Request(MessageRequest),
    Response(MessageResponse),
}

impl Message {
    pub fn new_request(cmd: MessageRequestCommand, if_index: u32) -> io::Result<Self> {
        Ok(Self::Request(MessageRequest {
            cmd,
            if_index: MeshIfIndex(if_index),
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
    fn parse_with_param(buffer: &[u8], header: GenlHeader) -> Result<Self, DecodeError> {
        MessageResponse::parse_with_param(buffer, header).map(Self::Response)
    }
}
