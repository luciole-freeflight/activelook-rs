//! ActiveLook Protocol
//!
//! Command packet
//! There are two types of packets :
//! - with 1 byte length field
//! - with 2 bytes length field
//!
//! optionally, the packets can have a query_id field.
//! The size is defined in the command_format field.
//!
//!
//! | 0xFF   | 0x..       | 0x0n           | 0x..        | n * 0x…   | m * 0x…        | 0xAA   |
//! |--------|------------|----------------|-------------|-----------|----------------|--------|
//! | Start  | Command ID | Command Format | Length      | Query ID  | Data           | Footer |
//! | 1B     | 1B         | 1B             | 1B          | nB        | mB             | 1B     |
//! |--------|------------|----------------|-------------|-----------|----------------|--------|
//! | Marker | Application| Protocol       | Protocol    | Protocol  | Application    | Marker |
//!
//!
//! | 0xFF   | 0x..       | 0x1n           | 0x.. 0x..   | n * 0x…  | m * 0x…        | 0xAA   |
//! |--------|------------|----------------|-------------|----------|----------------|--------|
//! | Start  | Command ID | Command Format | Length      | Query ID | Data           | Footer |
//! | 1B     | 1B         | 1B             | 2B          | nB       | mB             | 1B     |
//!
use crate::{
    commands::{self, Command, Response},
    traits::*,
};
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;
use log::*;
use thiserror::Error;

pub const PACKET_MIN_SIZE: usize = 5;
const PACKET_START: u8 = 0xFF;
const PACKET_END: u8 = 0xAA;

#[derive(Error, Debug, PartialEq)]
pub enum ProtocolError {
    #[error("Packet length is too small to contain a valid packet")]
    PacketLengthTooSmall,
    #[error("Packet delimiters are incorrect")]
    FrameError,
    #[error("Invalid packet length")]
    InvalidPacketLength,
    #[error(transparent)]
    ParseError(#[from] DekuError),
}

#[deku_derive(DekuRead, DekuWrite)]
pub struct CmdFormat {
    #[deku(bits = "3")]
    _reserved: u8,
    #[deku(bits = "1")]
    pub long: u8,
    #[deku(bits = "4")]
    pub query_id_size: usize,
}

impl Default for CmdFormat {
    fn default() -> Self {
        Self {
            _reserved: 0,
            long: 0,
            query_id_size: 0,
        }
    }
}

/// An ActiveLook BLE packet
pub struct Packet<T> {
    cmd_id: u8,
    format: CmdFormat,
    length: i16,
    query_id: Option<Vec<u8>>,
    /// Contains the application payload: [Command] or [Response]
    data: T,
}

/// Packet containing raw bytes
pub type RawPacket<'a> = Packet<Option<&'a [u8]>>;

/// Packet embedding a [Command]
pub type CommandPacket = Packet<Command>;

/// Packet embedding a [Response]
pub type ResponsePacket = Packet<Response>;

impl<'a> RawPacket<'a> {
    /// Construct a Packet from raw bytes
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ProtocolError> {
        if data.len() < PACKET_MIN_SIZE {
            return Err(ProtocolError::PacketLengthTooSmall);
        }

        if data.first() != Some(&PACKET_START) || data.last() != Some(&PACKET_END) {
            return Err(ProtocolError::FrameError);
        }

        // Used to manually deserialize the packet
        let mut index: usize = 1;

        // Command ID
        let cmd_id = data[index];
        index += 1;

        // Command Format
        // from_bytes() takes the offset in bits, hence the * 8
        let (_, cmd_format) = CmdFormat::from_bytes((data, index * 8))?;
        index += 1;

        // Length
        // Total length of the packet, including the start and stop delimiters.
        let length: i16 = if cmd_format.long == 1 {
            let len = i16::from_be_bytes(data[index..index + 1].try_into().unwrap());
            index += 2;
            len
        } else {
            let len = data[index];
            index += 1;
            len as i16
        };

        if data.len() != length as usize {
            return Err(ProtocolError::InvalidPacketLength);
        }

        // QueryID
        let query_id = match cmd_format.query_id_size {
            0 => None,
            len => Some(Vec::from(&data[index..index + len as usize])),
        };
        index += cmd_format.query_id_size;

        // Data
        let data_len = length as usize
            -2 // delimiters 
            -1 // cmd_id
            -1 // cmd_format
            -cmd_format.query_id_size
            -1; // length

        let data = match data_len {
            0 => None,
            len => Some(&data[index..index + len]),
        };

        Ok(Packet {
            cmd_id,
            format: cmd_format,
            length,
            query_id,
            data,
        })
    }
}

impl CommandPacket {
    pub fn from_bytes<'a>(data: &'a [u8]) -> Result<Self, ProtocolError> {
        let raw = RawPacket::from_bytes(data)?;
        Ok(Self::from(raw))
    }
}

impl From<RawPacket<'_>> for CommandPacket {
    fn from(raw: RawPacket) -> Self {
        Self {
            cmd_id: raw.cmd_id,
            format: raw.format,
            length: raw.length,
            query_id: raw.query_id,
            data: Command::from_data(raw.cmd_id, raw.data).expect("Invalid command bytestream"),
        }
    }
}

impl ResponsePacket {
    pub fn from_bytes<'a>(data: &'a [u8]) -> Result<Self, ProtocolError> {
        let raw = RawPacket::from_bytes(data)?;
        Ok(Self::from(raw))
    }
}

impl From<RawPacket<'_>> for ResponsePacket {
    fn from(raw: RawPacket) -> Self {
        Self {
            cmd_id: raw.cmd_id,
            format: raw.format,
            length: raw.length,
            query_id: raw.query_id,
            data: Response::from_data(raw.cmd_id, raw.data).expect("Invalid response bytestream"),
        }
    }
}

impl<T> Packet<T>
where
    T: Serializable + Deserializable,
{
    /// Create a packet from a [Command] or [Response]
    pub fn new(from: T) -> Self {
        let mut cmd_format = CmdFormat::default();
        let mut length: i16 = from.data_bytes().expect("Should have data").len() as i16 + 5;
        if length > 255 {
            cmd_format.long = 1;
            length += 1;
        }
        Self {
            cmd_id: from.id().expect("Should be a valid Command"),
            format: cmd_format,
            length,
            query_id: None,
            data: from,
        }
    }

    /// Create a packet from a [Command] or [Response], with a given query_id
    pub fn new_with_query_id(from: T, query_id: &[u8]) -> Self {
        let mut packet = Packet::new(from);
        packet.query_id = Some(Vec::from(query_id));
        packet.format.query_id_size = query_id.len();
        packet.length += packet.format.query_id_size as i16;
        packet
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        res.push(0xFF);
        res.push(self.cmd_id);
        res.extend(self.format.to_bytes().unwrap());

        if self.length > 255 {
            res.extend(self.length.to_be_bytes());
        } else {
            res.push(self.length as u8);
        }

        if let Some(query) = &self.query_id {
            res.extend(query);
        }

        res.extend(self.data.data_bytes().expect("Should be able to unwrap"));
        res.push(0xAA);
        res
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_packet_too_small() {
        let data = [0xFF, 0xAA];
        assert_eq!(
            Some(ProtocolError::PacketLengthTooSmall),
            RawPacket::from_bytes(&data).err()
        );
    }

    #[test]
    fn test_packet_incorrect_length() {
        let data = [
            0xFF, // start
            0x01, // CmdID
            0x00, // CmdFormat
            0x42, // Incorrect length
            // No query ID
            // No data
            0xAA, // end
        ];
        assert_eq!(
            Some(ProtocolError::InvalidPacketLength),
            RawPacket::from_bytes(&data).err()
        );
    }

    #[test]
    fn test_raw_to_command_conversion_without_data() {
        let cmd = Command::Clear;
        let raw = RawPacket {
            cmd_id: cmd.id().unwrap(),
            format: CmdFormat::default(),
            length: 1,
            query_id: None,
            data: None,
        };

        let packet = CommandPacket::from(raw);
        assert_eq!(packet.cmd_id, 0x01);
        assert_eq!(packet.data, cmd);
    }

    #[test]
    fn test_raw_to_command_conversion_with_data() {
        let cmd = Command::PowerDisplay { en: 1 };
        let raw = RawPacket {
            cmd_id: cmd.id().unwrap(),
            format: CmdFormat::default(),
            length: 1,
            query_id: None,
            data: Some(&[0x01]),
        };

        let packet = CommandPacket::from(raw);
        assert_eq!(packet.cmd_id, 0x00);
        assert_eq!(packet.data, cmd);
    }

    #[test]
    fn test_packet_creation() {
        let cmd = Command::PowerDisplay { en: 1 };
        let packet = Packet::new(cmd);
        assert_eq!(packet.cmd_id, 0x00);
    }

    #[test]
    fn test_packet_serialization() {
        let expected = [0xFF, 0x00, 0x00, 0x06, 0x01, 0xAA];
        let expected_cmd = Command::PowerDisplay { en: 1 };
        let cmd = Command::PowerDisplay { en: 1 };
        let packet = Packet::new(cmd);
        // Serialization
        let bytes = packet.to_bytes();
        assert_eq!(expected, bytes[..]);

        // Deserialization
        let newpkt = CommandPacket::from_bytes(&bytes).expect("Should be able to deserialize");
        assert_eq!(expected_cmd, newpkt.data);
    }
}

use embedded_io::{Read, Write, WriteReady};

/// Flow Control: used to prevent the Client Device application from overloading the BLE memory
/// buffer of the ActiveLook device.
#[repr(u8)]
pub enum FlowErrorCtrl {
    // Flow control
    /// Client can send data
    ClientCanSend = 0x01,
    /// Buffer reaches 75%, the client should stop sending data and wait for value return to 0x01
    ClientShouldWait = 0x02,

    // Error control
    /// The command was incomplete or corrupt, the command is ignored
    MessageError = 0x03,
    /// Receive message queue overflow
    MessageQueueOverflow = 0x04,
    ReservedError = 0x05,
    /// Missing the `cfgWrite` command before configuration modification
    MissingCfgWrite = 0x06,
}

/// Client which uses:
/// - Connection to Tx Activelook Server (Notify)
/// - Connection to Rx Activelook Server (Write)
/// - Connection to Control server (Notify)
pub struct ActiveLookClient<TxActiveLook, RxActiveLook, Ctrl>
where
    TxActiveLook: Read,
    RxActiveLook: Write + WriteReady,
    Ctrl: Read,
{
    /// Client Rx is connected to ActiveLook Tx
    rx: TxActiveLook,
    /// Client Tx is connected to ActiveLook Rx
    tx: RxActiveLook,
    ctrl: Ctrl,
}

/// Protocol implementation
/// https://github.com/ActiveLook/Activelook-API-Documentation/blob/fw-4.12.0_doc-revA/ActiveLook_API.md#35-control-server
impl<TxActiveLook, RxActiveLook, Ctrl> ActiveLookClient<TxActiveLook, RxActiveLook, Ctrl>
where
    TxActiveLook: Read,
    RxActiveLook: Write + WriteReady,
    Ctrl: Read,
{
}
