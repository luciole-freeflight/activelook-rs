use bitvec::prelude::*;
/// ActiveLook commands
///
///
///
/// ActiveLook Command packet
/// There are two types of packets :
/// - with 1 byte length field
/// - with 2 bytes length field
///
/// optionally, the packets can have a query_id field.
/// The size is defined in the command_format field.
///
///
/// | 0xFF   | 0x..       | 0x0n           | 0x..        | n * 0x…   | m * 0x…        | 0xAA   |
/// |--------|------------|----------------|-------------|-----------|----------------|--------|
/// | Start  | Command ID | Command Format | Length      | Query ID  | Data           | Footer |
/// | 1B     | 1B         | 1B             | 1B          | nB        | mB             | 1B     |
/// |--------|------------|----------------|-------------|-----------|----------------|--------|
/// |   -    | X          | -              | -           | X         | X              | -      |
///
///
/// | 0xFF   | 0x..       | 0x1n           | 0x.. 0x..   | n * 0x…  | m * 0x…        | 0xAA   |
/// |--------|------------|----------------|-------------|----------|----------------|--------|
/// | Start  | Command ID | Command Format | Length      | Query ID | Data           | Footer |
/// | 1B     | 1B         | 1B             | 2B          | nB       | mB             | 1B     |
///
///
/// An application only needs the following:
/// - Command ID
/// - Query ID
/// - Data
///
/// The rest can safely be ignored in the application, and computed on the fly during
/// serialization.
///
/// This seems complicated / impossible to serialize only with deku attributes, so we need a
/// hand-crafted top-level API.
///
use thiserror::Error;
// We use [deku](https://docs.rs/deku) for now.
// TODO Try [binrw](https://binrw.rs/)
use deku::prelude::*;

#[derive(Error, Debug)]
pub enum ActiveLookError {
    #[error("Incorrectly delimited bytestream (expected 0xFF ... 0xAA)")]
    DelimiterError,
    #[error("Unable to parse")]
    ParsingError(#[from] DekuError),
    #[error("Buffer too small")]
    SizeError,
    #[error("Unknown error")]
    UnknownError,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Eq, PartialEq)]
pub struct CommandFormat {
    #[deku(pad_bits_before = "3", bits = 1)]
    big_len: bool,
    #[deku(bits = 4)]
    query_id_len: u8,
}

/*
#[derive(Debug, Eq, PartialEq)]
#[deku_derive(DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct MasterToActiveLookCommand {
    #[deku(update = "self.data.deku_id()")]
    cmd_id: u8,

    #[deku(
        update = "MasterToActiveLookCommand::update_format( *self.length, self.query_id.len())"
    )]
    cmd_format: CommandFormat,

    #[deku(
        reader = "MasterToActiveLookCommand::read_len(deku::rest, cmd_format.big_len)",
        writer = "MasterToActiveLookCommand::write_len(deku::output, *length)"
    )]
    length: u16,

    #[deku(count = "cmd_format.query_id_len")]
    query_id: Vec<u8>,

    #[deku(ctx = "*cmd_id, (*length - 5 - query_id.len() as u16)")]
    data: MasterToActiveLookData,
}

impl MasterToActiveLookCommand {
    /// Decode an ActiveLook-delimited buffer into a Command
    fn decode(bytes: &[u8]) -> Result<Self, ActiveLookError> {
        if let Some((start, rest)) = bytes.split_first() {
            if start != &0xFF {
                return Err(ActiveLookError::DelimiterError);
            }
            if let Some((end, rest)) = rest.split_last() {
                if end != &0xAA {
                    return Err(ActiveLookError::DelimiterError);
                }

                if let Ok((_rest, cmd)) = MasterToActiveLookCommand::from_bytes((rest, 0)) {
                    return Ok(cmd);
                }
            }
        }
        Err(ActiveLookError::DelimiterError)
    }

    /*
    fn encode(self) -> Vec<u8> {
        let mut bytes = BitVec::new();
        self.data.to_bytes()
    }
    */
    /// Write cmd_format
    fn update_format(len: u16, query_len: u8) -> CommandFormat {
        CommandFormat {
            big_len: (len > 255),
            query_id_len: query_len,
        }
    }

    /// Read the viariable-size length field
    fn read_len(
        rest: &BitSlice<u8, Msb0>,
        big_len: bool,
    ) -> Result<(&BitSlice<u8, Msb0>, u16), DekuError> {
        let (rest, value) = if big_len {
            u16::read(rest, ())?
        } else {
            let (rest, u8value) = u8::read(rest, ())?;
            (rest, u8value as u16)
        };
        Ok((rest, value))
    }

    /// Write the variable-size length field
    fn write_len(output: &mut BitVec<u8, Msb0>, length: u16) -> Result<(), DekuError> {
        if length > 255 {
            length.write(output, ())
        } else {
            let u8length = length as u8;
            u8length.write(output, ())
        }
    }
}
*/
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Eq, PartialEq)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum LedState {
    #[deku(id = "0x00")]
    Off,
    #[deku(id = "0x01")]
    On,
    #[deku(id = "0x02")]
    Toggle,
    #[deku(id = "0x03")]
    Blinking,
}

#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum DeviceInfo {
    #[deku(id = "0")]
    HWPlatform,
    #[deku(id = "1")]
    Manufacturer,
    #[deku(id = "2")]
    AdvertisingManufacturerID,
    #[deku(id = "3")]
    Model,
    #[deku(id = "4")]
    SubModel,
    #[deku(id = "5")]
    FWVersion,
    #[deku(id = "6")]
    SerialNumber,
    #[deku(id = "7")]
    BatteryModel,
    #[deku(id = "8")]
    LensModel,
    #[deku(id = "9")]
    DisplayModel,
    #[deku(id = "10")]
    DisplayOrientation,
    #[deku(id = "11")]
    Certification1,
    #[deku(id = "12")]
    Certification2,
    #[deku(id = "13")]
    Certification3,
    #[deku(id = "14")]
    Certification4,
    #[deku(id = "15")]
    Certification5,
    #[deku(id = "16")]
    Certification6,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Eq, PartialEq)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum CmdError {
    #[deku(id = "1")]
    Generic,
    /// Missing the `cgfWrite` command before configuration modification
    #[deku(id = "2")]
    MissingCfgWrite,
    /// Memory read/write error
    #[deku(id = "3")]
    MemoryAccess,
    /// Protocol decoding error
    #[deku(id = "4")]
    ProtocolDecoding,
}

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Eq, PartialEq)]
#[deku(ctx = "cmd_id: u8, length: u16", id = "cmd_id")]
#[repr(u8)]
pub enum MasterToActiveLookData {
    ///
    /// Enable / disable power of the display
    #[deku(id = "0x00")]
    DisplayPower { en: bool },
    /// Clear the display memory (black screen)
    #[deku(id = "0x01")]
    Clear,
    /// Set the whole display to the corresponding grey level (0 to 15)
    #[deku(id = "0x02")]
    Grey { lvl: u8 },
    /// Display demonstration
    #[deku(id = "0x03")]
    Demo { demo_id: u8 },
    /// Get the battery level in %
    #[deku(id = "0x05")]
    Battery,
    /// Get the device ID and firmware version
    #[deku(id = "0x06")]
    Vers,
    /// Set green LED
    #[deku(id = "0x08")]
    Led { state: LedState },
    /// Shift all subsequently displayed objects of (x, y) pixels.
    #[deku(id = "0x09")]
    Shift { x: i16, y: i16 },
    /// Return the user parameters (shift, luma, sensor)
    #[deku(id = "0x0a")]
    Settings,
    /// Set the display luminance to the corresponding level (0 to 15)
    #[deku(id = "0x10")]
    Luma { level: u8 },

    ///
    /// Turn on/off the auto-brightness adjustment and gesture detection.
    #[deku(id = "0x20")]
    Sensor { en: bool },
    /// Turn on/off the gesture detection only
    #[deku(id = "0x21")]
    Gesture { en: bool },
    /// Turn on/off the auto-brightness adjustment only
    #[deku(id = "0x22")]
    Als { en: bool },

    ///
    /// Set the grey level (0 to 15) used to draw the next graphical element
    #[deku(id = "0x30")]
    Color { color: u8 },
    /// Set a pixel on at the corresponding coordinates
    #[deku(id = "0x31")]
    Point { x: i16, y: i16 },
    /// Draw a line at the corresponding coordinates
    #[deku(id = "0x32")]
    Line { x0: i16, y0: i16, x1: i16, y1: i16 },
    /// Draw an empty rectangle at the corresponding coordinates
    #[deku(id = "0x33")]
    Rect { x0: i16, y0: i16, x1: i16, y1: i16 },
    /// Draw a full rectangle at the corresponding coordinates
    #[deku(id = "0x34")]
    RectF { x0: i16, y0: i16, x1: i16, y1: i16 },
    /// Draw an empty circle at the corresponding coordinates
    #[deku(id = "0x35")]
    Circ { x: i16, y: i16, r: u8 },
    /// Draw a full circle at the corresponding coordinates
    #[deku(id = "0x36")]
    CircF { x: i16, y: i16, r: u8 },
    /// Write text `string` at coordinates (x, y) with rotation, font size and color
    #[deku(id = "0x37")]
    Txt {
        x: i16,
        y: i16,
        rotation: u8,
        font_size: u8,
        color: u8,
        string: [u8; 255],
    },
    /// Draw multiple connected lines at the corresponding coordinates.
    /// Size: 3 + (n+1) * 4
    /// NOT IMPLEMENTED (see variable size)
    #[deku(id = "0x38")]
    Polyline,
    /// Hold or flush the graphic engine.
    /// When held, new display commands are stored in memory and are displayed when the graphic engine is flushed.
    /// This allows stacking multiple graphic operations and displaying them simultaneously without screen flickering.
    /// The command is nested, the flush must be used the same number of times the hold was used
    /// action = 0 : Hold display
    /// action = 1 : Flush display
    /// action = 0xFF : Reset and flush all stacked hold. To be used when the state of the device is unknown
    /// After a BLE disconnect or an overflow error graphic engine is reset and flushed
    #[deku(id = "0x39")]
    HoldFlush { action: u8 },
    /// Draw an arc circle at the corresponding coordinates.
    /// Angles are in degrees, begin at 3 o'clock, and increase clockwise.
    #[deku(id = "0x3C")]
    Arc {
        x: i16,
        y: i16,
        r: u8,
        angle_start: i16,
        angle_end: i16,
        thickness: u8,
    },

    ///
    /// Save an image of `size` bytes and `width` pixels.
    /// Save image according to `format`:
    /// - 0x00: 4bpp
    /// - 0x01: 1bpp, transformed into 4bpp by the firmware before saving
    /// - 0x02: 4bpp with Heatshrink compression, decompressed into 4bpp by the firmware before saving
    /// - 0x03: 4bpp with Heatshrink compression, stored compressed, decompressed into 4bpp before display
    /// - 0x08: 8bpp with 4 bits for grey level and 4 bits for alpha channel
    #[deku(id = "0x41")]
    Save {
        id: u8,
        size: u32,
        width: u16,
        format: u8,
    },
    /// Display image `id` to the corresponding coordinates.
    /// Coordinates are signed, they can be negative.
    #[deku(id = "0x42")]
    Display { id: u8, x: i16, y: i16 },
    /// Stream an image on display without saving it in memory.
    /// Supported formats:
    /// - 0x01: 1bpp
    /// - 0x02: 4bpp with Heatshrink compression
    #[deku(id = "0x44")]
    Stream {
        size: u32,
        width: u16,
        x: i16,
        y: i16,
        format: u8,
    },
    /// Delete image.
    /// If `id` = 0xFF, delete all images.
    #[deku(id = "0x46")]
    Delete { id: u8 },
    /// Give the list of saved images.
    #[deku(id = "0x47")]
    List,

    ///
    /// Shutdown the device. The key must be equal to `0x6f 0x7f 0xc4 0xee`
    /// Shutdown is **NOT** allowed while USB powered.
    #[deku(id = "0xE0")]
    Shutdown { key: [u8; 4] },
    /// Reset the device. The key must be equal to `0x5c 0x1e 0x2d 0xe9`
    /// Reset is allowed **only** while USB powered.
    #[deku(id = "0xE1")]
    Reset { key: [u8; 4] },
    /// Read a device information parameter.
    #[deku(id = "0xE3")]
    Info { id: DeviceInfo },
}

impl MasterToActiveLookData {
    /// Extract useful bytes from a delimited bytestream
    fn remove_delimiters(bytes: &[u8]) -> Result<&[u8], ActiveLookError> {
        if let Some((start, rest)) = bytes.split_first() {
            if start != &0xFF {
                return Err(ActiveLookError::DelimiterError);
            }
            if let Some((end, rest)) = rest.split_last() {
                if end != &0xAA {
                    return Err(ActiveLookError::DelimiterError);
                }
                return Ok(rest);
            }
        }
        Err(ActiveLookError::DelimiterError)
    }

    /// Deserialize
    fn from(bytes: &[u8]) -> Result<Self, ActiveLookError> {
        let inner = MasterToActiveLookData::remove_delimiters(bytes)?;
        // Read CommandID
        let (cmd_id, rest) = inner.split_first().ok_or(ActiveLookError::SizeError)?;
        // Read CommandFormat
        let (rest, format) = CommandFormat::from_bytes((rest, 0))?;
        // Read variable size length
        let (rest, length) = if format.big_len {
            let value: u16 = (rest.0[rest.1] as u16) << 8 + rest.0[rest.1 + 1];
            ((rest.0, rest.1 + 16), value)
        } else {
            let value: u16 = rest.0[rest.1] as u16;
            ((rest.0, rest.1 + 8), value)
        };
        todo!();
        Err(ActiveLookError::UnknownError)
    }
}

#[derive(DekuRead, DekuWrite, Debug, Eq, PartialEq)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum ActiveLookToMaster {
    ///
    /// Battery level in % (0x64 = 100%)
    #[deku(id = "0x05")]
    Battery { level: u8 },
    /// Device ID and firmware version
    #[deku(id = "0x06")]
    Vers {
        /// fw version format: 3.5.0b = 0x03 0x05 0x00 0x62
        fw_version: [u8; 4],
        /// Manufacturing year
        mfc_year: u8,
        /// Manufacturing week
        mfc_week: u8,
        /// serial number example: 0x00 0x00 0x02
        serial_number: [u8; 3],
    },
    /// User parameters
    #[deku(id = "0x0A")]
    Settings {
        /// Global X shift
        x: i8,
        /// Global Y shift
        y: i8,
        /// display luminance (0 to 15)
        luma: u8,
        /// auto-brightness adjustment status
        als_enable: bool,
        /// gesture deteection status
        gesture_enable: bool,
    },

    ///
    /// List of images in memory.
    /// `height` and `width` are in pixels.
    /// Listing is not sorted.
    /// NOT IMPLEMENTED (see variable size)
    #[deku(id = "0x47")]
    ImgList,

    ///
    /// This message is sent asynchronously when there is an error during command processing.
    /// `cmd_id` is the ID of the command who got an error.
    #[deku(id = "0xE2")]
    CmdError {
        cmd_id: u8,
        error: u8,
        sub_error: u8,
    },
    /// Device parameter value.
    /// Size depends on parameter and its value
    /// TODO
    #[deku(id = "0xE3")]
    DevInfo { parameter: u8 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_format() {
        let bytes = [0b000_1_0011u8];
        let expected = CommandFormat {
            big_len: true,
            query_id_len: 3,
        };

        let (rest, decoded) = CommandFormat::from_bytes((&bytes[..], 0)).unwrap();

        assert_eq!(expected, decoded);
    }

    #[test]
    fn test_full_command_decoding() {
        let bytes = [0xFF, 0x00, 0x00, 0x06, 0x01, 0xAA];
        let expected = MasterToActiveLookData::DisplayPower { en: true };
        /*
        let cmd = MasterToActiveLookCommand::decode(&bytes).unwrap();
        assert_eq!(0x00, cmd.cmd_id);
        assert_eq!(0x06, cmd.length);
        assert_eq!(expected, cmd.data);
        */
    }
}
