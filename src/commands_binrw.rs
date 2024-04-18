//!
//! We want an easy mapping between the bytes we send/receive to the glasses, and the logical
//! representation in Rust.
//!
//! ActiveLook Command packet
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
//! |--------|------------|----------------|-------------|-----------|----------------|--------|
//! |   -    | X          | -              | -           | X         | X              | -      |
//!
//!
//! | 0xFF   | 0x..       | 0x1n           | 0x.. 0x..   | n * 0x…  | m * 0x…        | 0xAA   |
//! |--------|------------|----------------|-------------|----------|----------------|--------|
//! | Start  | Command ID | Command Format | Length      | Query ID | Data           | Footer |
//! | 1B     | 1B         | 1B             | 2B          | nB       | mB             | 1B     |
//!
//! We could use Enums, but when serializing the discriminant is put immediately before the data.
//!
//! In ActiveLook protocol, this is not the case:
//! - The Enum discriminant corresponds to Command ID.
//! - The Enum data lives after the protocol encoding (format, length, etc.)
//!
//! In other terms, the useful payload is split in two. Classic de/serialization crates like
//! `binrw`, `deku` and so on can not do this in a simple way.
//!
//! So we will use:
//! - a unit-only enum for CommandID
//! - associated Structs for relevant Data
//! - a lower-level protocol handling the serialization, Query ID etc.
//!
use binrw::{binrw, io::Cursor, BinRead, BinWrite};
use thiserror::Error;

// ---------------------------------------------------------------------------
// All commands
// ---------------------------------------------------------------------------

#[binrw]
#[brw(repr(u8))]
#[derive(Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DemoID {
    Fill = 0,
    Rect = 1,
    Images = 2,
}

#[binrw]
#[brw(repr(u8))]
#[derive(Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum LedState {
    Off = 0,
    On = 1,
    Toggle = 2,
    Blinking = 3,
}

/// We HAVE TO duplicate discriminants and dinrw magic :(
#[binrw]
#[derive(Debug, Eq, PartialEq)]
#[brw(big)]
#[repr(u8)]
pub enum Command {
    #[brw(magic = 0x00u8)]
    PowerDisplay { en: u8 } = 0x00,
    #[brw(magic = 0x01u8)]
    Clear = 0x01,
    #[brw(magic = 0x02u8)]
    Grey { lvl: u8 } = 0x02,
    #[brw(magic = 0x03u8)]
    Demo { demo_id: DemoID } = 0x03,
    #[brw(magic = 0x05u8)]
    Battery = 0x05,
    #[brw(magic = 0x06u8)]
    Version = 0x06,
    #[brw(magic = 0x08u8)]
    Led { state: LedState } = 0x08,
    #[brw(magic = 0x09u8)]
    Shift { x: i16, y: i16 } = 0x09,
    #[brw(magic = 0x0Au8)]
    Settings = 0x0A,
}

impl Command {
    /// Access the discriminant as unique ID
    /// <https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting>
    pub fn id(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

// ---------------------------------------------------------------------------
// All responses
// ---------------------------------------------------------------------------
#[derive(Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Response {
    Battery {
        level: u8,
    } = 0x05,
    Version {
        fw_version: [u8; 4],
        mfc_year: u8,
        mfc_week: u8,
        serial_number: [u8; 3],
    } = 0x06,
    Settings {
        x: i8,
        y: i8,
        luma: u8,
        als_enable: u8,
        gesture_enable: u8,
    } = 0x0A,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id() {
        assert_eq!(0, Command::PowerDisplay { en: true as u8 }.id());
        assert_eq!(1, Command::Clear.id());
        assert_eq!(0x0A, Command::Settings.id());
    }

    #[test]
    fn test_serialization() {
        let mut writer = Cursor::new(Vec::new());
        let expected: &[u8] = &[0x00, 0x01];
        let cmd = Command::PowerDisplay { en: true as u8 };
        cmd.write(&mut writer).unwrap();
        assert_eq!(expected, writer.get_ref());
    }
}
