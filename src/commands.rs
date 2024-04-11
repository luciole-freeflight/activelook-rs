use binary_layout::prelude::*;
/// ActiveLook commands
use deku::prelude::*;

#[derive(DekuRead, DekuWrite, Debug, Eq, PartialEq)]
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

#[derive(DekuRead, DekuWrite, Debug, Eq, PartialEq)]
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

#[derive(DekuRead, DekuWrite, Debug, Eq, PartialEq)]
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

#[derive(DekuRead, DekuWrite, Debug, Eq, PartialEq)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum MasterToActiveLook {
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
        // string: [u8; 255], // XXX the trait `general::_::_serde::Deserialize<'_>` is not implemented for `[u8; 255]55]`
        string: [u8; 32],
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

mod test_binary_layout {
    use binary_layout::prelude::*;

    binary_layout!(graphics_txt, BigEndian, {
        x: i16,
        y: i16,
        r: u8,
        f: u8,
        c: u8,
        string: [u8], // open ended byte array, matches until the end of the packet
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding() {
        let cmd = MasterToActiveLook::DisplayPower { en: true };
        let expected: &[u8] = &[0, 1];

        let bytes = cmd.to_bytes().unwrap();
        assert_eq!(expected, bytes);

        let decoded = MasterToActiveLook::try_from(expected).unwrap();
        assert_eq!(decoded, cmd);
    }

    #[test]
    fn test_binary_layout_txt() {
        let bytes = &[0, 0, 0, 0, 0, 8, 42, 0x30, 0x31, 0x32, 0];
        let view = test_binary_layout::graphics_txt::View::new(bytes);

        let mut memory = [0u8; 255];

        println!("Size {:?}", test_binary_layout::graphics_txt::SIZE);
        let mut expected = test_binary_layout::graphics_txt::View::new(memory);
        expected.x_mut().write(0);
        expected.y_mut().write(0);
        expected.r_mut().write(0);
        expected.f_mut().write(8);
        expected.c_mut().write(42);
        expected.string_mut().copy_from_slice(b"123\0");

        assert_eq!(bytes, &memory[..bytes.len()]);
    }
}
