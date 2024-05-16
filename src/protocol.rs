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

pub struct Packet<T> {
    cmd_id: u8,
    format: CmdFormat,
    length: i16,
    query_id: Option<Vec<u8>>,
    data: T,
}

pub type RawPacket = Packet<Option<&'static [u8]>>;
pub type CommandPacket = Packet<Command>;
pub type ResponsePacket = Packet<Command>;

impl RawPacket {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProtocolError> {
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
        let mut length: i16 = if cmd_format.long == 1 {
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
        let data = None;

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
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProtocolError> {
        let raw = RawPacket::from_bytes(data)?;
        todo!()
    }
}

impl From<RawPacket> for CommandPacket {
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
    fn test_raw_to_command_conversion() {
        let cmd = Command::Clear;
        let raw = RawPacket {
            cmd_id: cmd.id().unwrap(),
            format: CmdFormat::default(),
            length: 1,
            query_id: None,
            data: None,
        };

        let packet = CommandPacket::from(raw);
    }
}

// ---------------------------------------------------------------------------
//pub fn encode(cmd: &Command)
