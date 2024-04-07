use heapless::Vec;
use postcard::{from_bytes, to_vec};
/// ActiveLook commands
///
use serde::{Deserialize, Serialize};

pub mod general {
    use super::*;

    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum LedState {
        Off = 0,
        On = 1,
        Toggle = 2,
        Blinking = 3,
    }

    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {
        /// Enable / disable power of the display
        DisplayPower { en: bool } = 0x00,
        /// Clear the display memory (black screen)
        Clear = 0x01,
        /// Set the whole display to the corresponding grey level (0 to 15)
        Grey { lvl: u8 } = 0x02,
        /// Display demonstration
        Demo { demo_id: u8 } = 0x03,
        /// Get the battery level in %
        Battery = 0x05,
        /// Get the device ID and firmware version
        Vers = 0x06,
        /// Set green LED
        Led { state: LedState } = 0x08,
        /// Shift all subsequently displayed objects of (x, y) pixels.
        Shift { x: i16, y: i16 } = 0x09,
        /// Return the user parameters (shift, luma, sensor)
        Settings = 0x0a,
        /// Set the display luminance to the corresponding level (0 to 15)
        Luma { level: u8 } = 0x10,
    }

    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {
        /// Battery level in % (0x64 = 100%)
        Battery { level: u8 } = 0x05,
        /// Device ID and firmware version
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
    }
}

pub mod optical {
    use super::*;
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {
        /// Turn on/off the auto-brightness adjustment and gesture detection.
        Sensor { en: bool } = 0x20,
        /// Turn on/off the gesture detection only
        Gesture { en: bool } = 0x21,
        /// Turn on/off the auto-brightness adjustment only
        Als { en: bool } = 0x22,
    }
}

pub mod graphics {
    use super::*;
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {
        /// Set the grey level (0 to 15) used to draw the next graphical element
        Color { color: u8 } = 0x30,
        /// Set a pixel on at the corresponding coordinates
        Point { x: i16, y: i16 } = 0x31,
        /// Draw a line at the corresponding coordinates
        Line { x0: i16, y0: i16, x1: i16, y1: i16 } = 0x32,
        /// Draw an empty rectangle at the corresponding coordinates
        Rect { x0: i16, y0: i16, x1: i16, y1: i16 } = 0x33,
        /// Draw a full rectangle at the corresponding coordinates
        RectF { x0: i16, y0: i16, x1: i16, y1: i16 } = 0x34,
        /// Draw an empty circle at the corresponding coordinates
        Circ { x: i16, y: i16, r: u8 } = 0x35,
        /// Draw a full circle at the corresponding coordinates
        CircF { x: i16, y: i16, r: u8 } = 0x36,
        /// Write text `string` at coordinates (x, y) with rotation, font size and color
        Txt {
            x: i16,
            y: i16,
            rotation: u8,
            font_size: u8,
            color: u8,
            // string: [u8; 255], // XXX the trait `general::_::_serde::Deserialize<'_>` is not implemented for `[u8; 255]55]`
            string: [u8; 32],
        } = 0x37,
        /// Draw multiple connected lines at the corresponding coordinates.
        /// Size: 3 + (n+1) * 4
        /// NOT IMPLEMENTED (see variable size)
        Polyline {} = 0x38,
        /// Hold or flush the graphic engine.
        /// When held, new display commands are stored in memory and are displayed when the graphic engine is flushed.
        /// This allows stacking multiple graphic operations and displaying them simultaneously without screen flickering.
        /// The command is nested, the flush must be used the same number of times the hold was used
        /// action = 0 : Hold display
        /// action = 1 : Flush display
        /// action = 0xFF : Reset and flush all stacked hold. To be used when the state of the device is unknown
        /// After a BLE disconnect or an overflow error graphic engine is reset and flushed
        HoldFlush { action: u8 } = 0x39,
        /// Draw an arc circle at the corresponding coordinates.
        /// Angles are in degrees, begin at 3 o'clock, and increase clockwise.
        Arc {
            x: i16,
            y: i16,
            r: u8,
            angle_start: i16,
            angle_end: i16,
            thickness: u8,
        } = 0x3C,
    }
}

pub mod image {
    use super::*;
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {
        /// Save an image of `size` bytes and `width` pixels.
        /// Save image according to `format`:
        /// - 0x00: 4bpp
        /// - 0x01: 1bpp, transformed into 4bpp by the firmware before saving
        /// - 0x02: 4bpp with Heatshrink compression, decompressed into 4bpp by the firmware before saving
        /// - 0x03: 4bpp with Heatshrink compression, stored compressed, decompressed into 4bpp before display
        /// - 0x08: 8bpp with 4 bits for grey level and 4 bits for alpha channel
        Save {
            id: u8,
            size: u32,
            width: u16,
            format: u8,
        } = 0x41,
        /// Display image `id` to the corresponding coordinates.
        /// Coordinates are signed, they can be negative.
        Display { id: u8, x: i16, y: i16 } = 0x42,
        /// Stream an image on display without saving it in memory.
        /// Supported formats:
        /// - 0x01: 1bpp
        /// - 0x02: 4bpp with Heatshrink compression
        Stream {
            size: u32,
            width: u16,
            x: i16,
            y: i16,
            format: u8,
        } = 0x44,
        /// Delete image.
        /// If `id` = 0xFF, delete all images.
        Delete { id: u8 } = 0x46,
        /// Give the list of saved images.
        List = 0x47,
    }

    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {
        /// List of images in memory.
        /// `height` and `width` are in pixels.
        /// Listing is not sorted.
        /// NOT IMPLEMENTED (see variable size)
        List = 0x47,
    }
}

pub mod font {
    use super::*;
    /*
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {}
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {}
    */
}

pub mod layout {
    use super::*;
    /*
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {}
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {}
    */
}

pub mod gauge {
    use super::*;
    /*
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {}
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {}
    */
}

pub mod page {
    use super::*;
    /*
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {}
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {}
    */
}

pub mod animation {
    use super::*;
    /*
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {}
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {}
    */
}

pub mod statistics {
    use super::*;
    /*
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {
        /// Get the number of pixels activated on the display
        PixelCount = 0xA5,
    }
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {
        /// Number of pixels activated on the display
        PixelCount { count: u32 } = 0xA5,
    }
    */
}

pub mod configuration {
    use super::*;
    /*
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {}
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {}
    */
}

pub mod device {
    use super::*;
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum DeviceInfo {
        HWPlatform = 0,
        Manufacturer = 1,
        AdvertisingManufacturerID = 2,
        Model = 3,
        SubModel = 4,
        FWVersion = 5,
        SerialNumber = 6,
        BatteryModel = 7,
        LensModel = 8,
        DisplayModel = 9,
        DisplayOrientation = 10,
        Certification1 = 11,
        Certification2 = 12,
        Certification3 = 13,
        Certification4 = 14,
        Certification5 = 15,
        Certification6 = 16,
    }

    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum Error {
        Generic = 1,
        /// Missing the `cgfWrite` command before configuration modification
        MissingCfgWrite = 2,
        /// Memory read/write error
        MemoryAccess = 3,
        /// Protocol decoding error
        ProtocolDecoding = 4,
    }

    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum MasterToActiveLook {
        /// Shutdown the device. The key must be equal to `0x6f 0x7f 0xc4 0xee`
        /// Shutdown is **NOT** allowed while USB powered.
        Shutdown { key: [u8; 4] } = 0xE0,
        /// Reset the device. The key must be equal to `0x5c 0x1e 0x2d 0xe9`
        /// Reset is allowed **only** while USB powered.
        Reset { key: [u8; 4] } = 0xE1,
        /// Read a device information parameter.
        Info { id: DeviceInfo } = 0xE3,
    }
    #[repr(u8)]
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub enum ActiveLookToMaster {
        /// This message is sent asynchronously when there is an error during command processing.
        /// `cmd_id` is the ID of the command who got an error.
        /// ``
        Error {
            cmd_id: u8,
            error: u8,
            sub_error: u8,
        } = 0xE2,
        /// Device parameter value.
        /// Size depends on parameter and its value
        /// TODO
        Info { parameter: u8 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding() {
        let cmd = general::MasterToActiveLook::DisplayPower { en: true };
        let expected = [0, 1];
        let bytes: Vec<u8, 2> = to_vec(&cmd).unwrap();
        assert_eq!(expected, bytes);
    }
}
