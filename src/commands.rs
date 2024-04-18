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
//use binrw::{binrw, io::Cursor, BinRead, BinWrite};
use deku::prelude::*;
use thiserror::Error;

// ---------------------------------------------------------------------------
// All commands
// ---------------------------------------------------------------------------

#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum DemoID {
    #[deku(id = "0")]
    Fill = 0,
    #[deku(id = "1")]
    Rect = 1,
    #[deku(id = "2")]
    Images = 2,
}

#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum LedState {
    #[deku(id = "0")]
    Off = 0,
    #[deku(id = "1")]
    On = 1,
    #[deku(id = "2")]
    Toggle = 2,
    #[deku(id = "3")]
    Blinking = 3,
}

#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum Command {
    /// Enable / disable power of the display
    #[deku(id = "0x00")]
    PowerDisplay { en: u8 },
    /// Clear the display memory (black screen)
    #[deku(id = "0x01")]
    Clear,
    /// Set the whole display to the corresponding grey level (0 to 15)
    #[deku(id = "0x02")]
    Grey { lvl: u8 },
    /// Display demonstration
    #[deku(id = "0x03")]
    Demo { demo_id: DemoID } = 0x03,
    /// Get the battery level in %
    #[deku(id = "0x05")]
    Battery,
    /// Get the device ID and firmware version
    #[deku(id = "0x06")]
    Version,
    /// Set green LED
    #[deku(id = "0x08")]
    Led { state: LedState },
    /// Shift all subsequently displayed objects of (x, y) pixels.
    #[deku(id = "0x09")]
    Shift { x: i16, y: i16 },
    /// Return the user parameters (shift, luma, sensor)
    #[deku(id = "0x0A")]
    Settings,
}

impl Command {
    /// Access the discriminant as unique ID
    pub fn id(&self) -> Result<u8, DekuError> {
        self.deku_id()
        // <https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting>
        //unsafe { *(self as *const Self as *const u8) }
    }

    /// Access data bytes for serialization.
    /// This might become expensive but we'll deal with that later.
    pub fn data_bytes(&self) -> Result<Vec<u8>, DekuError> {
        let mut bytes: Vec<u8> = self.to_bytes()?;
        bytes.remove(0);
        Ok(bytes)
    }

    /// Create a Command from the CommandID and data.
    pub fn from_data_bytes(id: u8, data: &[u8]) -> Result<Self, DekuError> {
        let mut bytes = vec![id];
        bytes.extend_from_slice(&data);
        let (_rest, cmd) = Command::from_bytes((&bytes, 0))?;
        Ok(cmd)
    }

    /// Extract CommandID and data bytes from Command
    pub fn to_data_bytes(&self) -> Result<(u8, Vec<u8>), DekuError> {
        let data = self.data_bytes()?;
        Ok((self.id()?, data))
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
        assert_eq!(0, Command::PowerDisplay { en: true as u8 }.id().unwrap());
        assert_eq!(1, Command::Clear.id().unwrap());
        assert_eq!(0x0A, Command::Settings.id().unwrap());
    }

    #[test]
    fn test_serialization() {
        let expected: &[u8] = &[0x00, 0x01];
        let cmd = Command::PowerDisplay { en: true as u8 };
        let bytes = cmd.to_bytes().unwrap();
        assert_eq!(expected, bytes);

        let data = cmd.data_bytes().unwrap();
        assert_eq!(expected[1..], data);

        let other = Command::from_data_bytes(0x00, &[0x01]).unwrap();
        assert_eq!(cmd, other);
    }
}
