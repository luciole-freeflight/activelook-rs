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
    commands::{Command, Response},
    traits::*,
};
use deku::prelude::*;
use embedded_io::{self, Read, Write};
//use embedded_io::{ReadReady, WriteReady};
use log::*;
use thiserror::Error;

/// Min packet size, based on the smallest valid packet
pub const PACKET_MIN_SIZE: usize = 5;
/// Max packet size, as defined in ActiveLook documentation 3.1. Rx Server - Length
pub const PACKET_MAX_SIZE: usize = 533;
/// Max data size, as defined in ActiveLook documentation 3.1. Rx Server - Length
pub const PACKET_DATA_MAX_SIZE: usize = 512;
/// Delimiter at the start of a packet
const PACKET_START: u8 = 0xFF;
/// Delimiter at the end of a packet
const PACKET_END: u8 = 0xAA;

/// Errors returned when dealing with the Protocol.
#[derive(Error, Debug, PartialEq)]
pub enum ProtocolError {
    /// The buffer is too small to contain a valid packet
    #[error("Packet length is too small to contain a valid packet")]
    PacketLengthTooSmall,
    /// The [Packet] is incorrectly delimited
    #[error("Packet delimiters are incorrect")]
    FrameError,
    /// The [Packet] length does not correspond to the buffer length
    #[error("Invalid packet length")]
    InvalidPacketLength,
    /// Error coming from [deku] serialization
    #[error(transparent)]
    ParseError(#[from] DekuError),
    /// [embedded_io::ErrorKind] coming from the underlying layer
    #[error("embedded_io::Error")]
    EmbeddedIOError,
    /// Incorrect QueryID
    #[error("QueryID does not correspond to sent Command")]
    IncorrectQueryId,
    /// Not an error, used to signify there is nothing to read
    #[error("No data")]
    Empty,
}

/// Some packet options
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Default)]
pub struct CmdFormat {
    #[deku(bits = "3")]
    _reserved: u8,
    /// The length field is on two bytes if true
    #[deku(bits = "1")]
    pub long: u8,
    /// Length of the QueryID field, if it exists
    #[deku(bits = "4")]
    pub query_id_size: usize,
}

/// An ActiveLook BLE packet
pub struct Packet<T> {
    cmd_id: u8,
    format: CmdFormat,
    length: i16,
    pub query_id: Option<Vec<u8>>,
    /// Contains the application payload: [Command] or [Response]
    pub data: T,
}

/// Packet containing raw bytes, to be interpreted later as [Command] or [Response]
pub type RawPacket<'a> = Packet<Option<&'a [u8]>>;

/// Packet embedding a [Command]
pub type CommandPacket = Packet<Command>;

/// Packet embedding a [Response]
pub type ResponsePacket = Packet<Response>;

impl<'a> RawPacket<'a> {
    /// Construct a Packet from raw bytes
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ProtocolError> {
        if bytes.len() < PACKET_MIN_SIZE {
            return Err(ProtocolError::PacketLengthTooSmall);
        }

        if bytes.first() != Some(&PACKET_START) || bytes.last() != Some(&PACKET_END) {
            return Err(ProtocolError::FrameError);
        }

        // Used to manually deserialize the packet
        let mut index: usize = 1;

        // Command ID
        let cmd_id = bytes[index];
        index += 1;

        // Command Format
        // from_bytes() takes the offset in bits, hence the * 8
        let (_, cmd_format) = CmdFormat::from_bytes((bytes, index * 8))?;
        index += 1;

        // Length
        // Total length of the packet, including the start and stop delimiters.
        let length: i16 = if cmd_format.long == 1 {
            let len = i16::from_be_bytes(bytes[index..index + 1].try_into().unwrap());
            index += 2;
            len
        } else {
            let len = bytes[index];
            index += 1;
            len as i16
        };

        if bytes.len() != length as usize {
            return Err(ProtocolError::InvalidPacketLength);
        }

        // QueryID
        let query_id = match cmd_format.query_id_size {
            0 => None,
            len => Some(Vec::from(&bytes[index..index + len])),
        };
        index += cmd_format.query_id_size;

        // Data
        let data_len = length as usize
            -2 // delimiters 
            -1 // cmd_id
            -1 // cmd_format
            -cmd_format.query_id_size
            -cmd_format.long as usize // -1 if length is on two bytes
            -1; // length

        let data = match data_len {
            0 => None,
            len => Some(&bytes[index..index + len]),
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
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let raw = RawPacket::from_bytes(bytes)?;
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
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let raw = RawPacket::from_bytes(bytes)?;
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
        let bytes = [0xFF, 0xAA];
        assert_eq!(
            Some(ProtocolError::PacketLengthTooSmall),
            RawPacket::from_bytes(&bytes).err()
        );
    }

    #[test]
    fn test_packet_incorrect_length() {
        let bytes = [
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
            RawPacket::from_bytes(&bytes).err()
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
    RxActiveLook: Write,
    Ctrl: Read,
{
    /// Client Rx is connected to ActiveLook Tx
    rx: TxActiveLook,
    /// Client Tx is connected to ActiveLook Rx
    tx: RxActiveLook,
    ctrl: Ctrl,
    /// Sequence number
    query_id: u32,
}

/// Protocol implementation
/// https://github.com/ActiveLook/Activelook-API-Documentation/blob/fw-4.12.0_doc-revA/ActiveLook_API.md#35-control-server
impl<TxActiveLook, RxActiveLook, Ctrl> ActiveLookClient<TxActiveLook, RxActiveLook, Ctrl>
where
    TxActiveLook: Read,
    RxActiveLook: Write,
    Ctrl: Read,
{
    pub fn new(rx: TxActiveLook, tx: RxActiveLook, ctrl: Ctrl) -> Self {
        Self {
            rx,
            tx,
            ctrl,
            query_id: 0,
        }
    }

    /// Send a command
    /// XXX This takes ownership of the Command for now
    pub fn send(&mut self, cmd: Command) -> Result<(), ProtocolError> {
        debug!("Sending command {:?}", &cmd);
        let packet = Packet::new_with_query_id(cmd, &self.query_id.to_be_bytes());
        self.query_id += 1;
        let res = self.tx.write(&packet.to_bytes()[..]);
        match res {
            Ok(_) => Ok(()),
            Err(error) => {
                error!("{:?}", error);
                Err(ProtocolError::EmbeddedIOError)
            }
        }
    }

    pub fn send_command_expect_response(
        &mut self,
        cmd: Command,
    ) -> Result<Response, ProtocolError> {
        debug!("Sending command {:?}, expecting Response", &cmd);
        let packet = Packet::new_with_query_id(cmd, &self.query_id.to_be_bytes());
        let res = self.tx.write(&packet.to_bytes()[..]);
        self.query_id += 1;
        if let Err(error) = res {
            return Err(ProtocolError::EmbeddedIOError);
        }

        let mut response_pkt: ResponsePacket;
        loop {
            let resp = self.read_tx_char();
            if let Ok(pkt) = resp {
                response_pkt = pkt;
                break;
            }
        }
        debug!("Received response {:?}", &response_pkt.data);
        if let Some(id) = response_pkt.query_id {
            if id.len() != core::mem::size_of::<u32>() {
                return Err(ProtocolError::IncorrectQueryId);
            }
            // Here unwrap() is safe, because we checked the vec length beforehand
            if u32::from_be_bytes(id.try_into().unwrap()) == self.query_id {
                Ok(response_pkt.data)
            } else {
                Err(ProtocolError::IncorrectQueryId)
            }
        } else {
            Err(ProtocolError::IncorrectQueryId)
        }
    }

    // Get notification on TX characteristic
    pub fn read_tx_char(&mut self) -> Result<ResponsePacket, ProtocolError> {
        let mut rxbuf = [0; PACKET_MAX_SIZE];
        if let Ok(len) = self.rx.read(&mut rxbuf) {
            ResponsePacket::from_bytes(&rxbuf[..len])
        } else {
            Err(ProtocolError::Empty)
        }
    }

    // Get notification on TX characteristic
    pub fn read_ctrl_char(&mut self) -> Result<u8, ProtocolError> {
        let mut rxbuf = [0; PACKET_MAX_SIZE];
        if let Ok(_len) = self.ctrl.read(&mut rxbuf) {
            Ok(rxbuf[0])
        } else {
            Err(ProtocolError::Empty)
        }
    }
}

/// Server which uses:
/// - Connection to Tx Activelook Server (Write)
/// - Connection to Rx Activelook Server (Notify)
/// - Connection to Control server (Write)
pub struct ActiveLookServer<TxActiveLook, RxActiveLook, Ctrl>
where
    TxActiveLook: Write,
    RxActiveLook: Read,
    Ctrl: Write,
{
    /// Server Rx is connected to ActiveLook Rx
    rx: RxActiveLook,
    /// Server Tx is connected to ActiveLook Tx
    tx: TxActiveLook,
    ctrl: Ctrl,
}

/// Protocol implementation
/// https://github.com/ActiveLook/Activelook-API-Documentation/blob/fw-4.12.0_doc-revA/ActiveLook_API.md#35-control-server
impl<TxActiveLook, RxActiveLook, Ctrl> ActiveLookServer<TxActiveLook, RxActiveLook, Ctrl>
where
    TxActiveLook: Write,
    RxActiveLook: Read,
    Ctrl: Write,
{
    pub fn new(rx: RxActiveLook, tx: TxActiveLook, ctrl: Ctrl) -> Self {
        Self { rx, tx, ctrl }
    }

    pub fn read_data(&mut self) -> Result<CommandPacket, ProtocolError> {
        let mut rxbuf = [0; PACKET_MAX_SIZE];
        if let Ok(len) = self.rx.read(&mut rxbuf) {
            CommandPacket::from_bytes(&rxbuf[..len])
        } else {
            //trace!("No data to read");
            Err(ProtocolError::Empty)
        }
    }

    pub fn send_response(&mut self, response: ResponsePacket) {
        let bytes = response.to_bytes();
        self.tx.write(&bytes);
    }
}
