//!
//! Access all ActiveLook commands and responses.
//!
//! We want an easy mapping between the bytes we send/receive to the glasses, and the logical
//! representation in Rust.
//!
//! We could use Enums, but when serializing the discriminant is put immediately before the data.
//!
//! In ActiveLook protocol, this is not the case:
//! - The Enum discriminant corresponds to Command ID.
//! - The Enum data lives after the protocol encoding (format, length, etc.)
//!
//! In other terms, the useful payload is split in two.
//! Classic de/serialization crates like `binrw`, `deku` and so on can not do this in a simple way.
//!
//! So we will use:
//! - `deku` Enums plus de/serialization traits and implementations
//! - a lower-level protocol handling the serialization, Query ID etc.
//!
//use binrw::{binrw, io::Cursor, BinRead, BinWrite};
use crate::traits::*;
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::ctx::BitSize;
use deku::prelude::*;
use deku::reader::Reader;
use thiserror::Error;

// ---------------------------------------------------------------------------
// All command and response items
// ---------------------------------------------------------------------------
/// Magic value denoting that ALL elements are concerned by the command
pub const ALL: u8 = 0xFF;

/// Max size for Layout names
pub const NAME_LEN: usize = 12;

/// Max size for free text
pub const TEXT_LEN: usize = 255;

/// Errors returned by ActiveLook glasses
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

/// Available Demo values for [Command::Demo]
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

/// Available state values for [Command::Led]
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

/// Available values for [Command::Info]
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

/// Hold or Flush the graphic engine.  
///
/// When held, new display commands are stored in memory and are displayed when the graphic engine
/// is flushed. This allows stacking multiple graphic operations and displaying them simultaneously
/// without screen flickering.  
/// The command is nested, the [HoldFlushAction::Flush] action must be used the same number of times
/// [HoldFlushAction::Hold] was used.
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum HoldFlushAction {
    /// Hold display
    #[deku(id = "0")]
    Hold,
    /// Flush display
    #[deku(id = "1")]
    Flush,
    /// Reset and flush all stacked hold. To be used when the state of the device is unknown.
    /// After a BLE disconnect or an overflow error, graphic engine is reset and flushed.
    #[deku(id = "255")]
    ResetFlush,
}

/// Common Point type used globally in commands
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

/// List item returned in [Response::ImgList]
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
pub struct ImgListItem {
    pub id: u8,
    pub height: u16,
    pub width: u16,
}

/// Font item used in [Response::FontList]
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
pub struct FontItem {
    pub id: u8,
    pub height: u8,
}

/// Configuration item used in [Response::CfgList]
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
pub struct CfgItem {
    /// Name of the configuration
    #[deku(
        reader = "read_fixed_size_cstr(deku::reader, NAME_LEN)",
        writer = "write_fixed_size_cstr(name, deku::output, NAME_LEN)"
    )]
    pub name: String,
    /// Size in bytes
    pub size: u32,
    /// Provided by user
    pub version: u32,
    /// Used to sort configurations, most recent used configuration have higher values
    pub usage_counter: u8,
    /// Used to sort configurations, most recent installed configuration have higher values
    pub install_counter: u8,
    /// Indicate system configuration, can't be deleted.
    pub is_system: u8,
}

/// Layout position item used in [Command::LayoutPosition] for instance
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
pub struct LayoutPosition {
    pub x: u16,
    pub y: u8,
}

/// Layout parameters
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
pub struct LayoutParameters {
    /// Size of additional commands in bytes
    size: u8,
    /// Upper left clipping region in the display
    pos: LayoutPosition,
    /// Width of the clipping region
    width: u16,
    /// Height of the clipping region
    height: u8,
    /// Foreground color (0..15)
    fore_color: u8,
    /// Background color (0..15)
    back_color: u8,
    font: u8,
    text_valid: u8,
    /// Test position in the clipping region
    text_pos: LayoutPosition,
    text_rotation: u8,
    /// If true, the background of each character should be drawn.
    /// Else, it leaves the background as is
    text_opacity: u8,
    /// Additional graphical commands
    #[deku(count = "size")]
    commands: Vec<u8>,
}
// ---------------------------------------------------------------------------
// Deku readers and writers
// ---------------------------------------------------------------------------
/// Read a fixed-len slice containing a 0-delimited C string.
/// The 0 is optional in the input if the max `len` is reached
fn read_fixed_size_cstr<R: deku::no_std_io::Read>(
    reader: &mut Reader<R>,
    len: usize,
) -> Result<String, DekuError> {
    let mut res = String::new();
    for _ in 0..len {
        let val = u8::from_reader_with_ctx(reader, BitSize(8))?;
        if val == '\0' as u8 {
            break;
        }
        res.push(val as char);
    }
    Ok(res)
}

fn write_fixed_size_cstr(
    string: &String,
    output: &mut BitVec<u8, Msb0>,
    len: usize,
) -> Result<(), DekuError> {
    let mut string = string.clone();
    string.truncate(len as usize);
    let s = string.as_bytes();
    s.write(output, BitSize(8))?;
    if s.len() < len {
        0u8.write(output, BitSize(8))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// All commands
// ---------------------------------------------------------------------------
/// These map to the commands MasterToActiveLook
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum Command {
    // --- General commands --
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
    Demo { demo_id: DemoID },
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

    // --- Luminance commands ---
    /// Set the display luminance to the corresponding level (0 to 15)
    #[deku(id = "0x10")]
    Luma { level: u8 },

    // --- Optical sensor commands ---
    /// Turn on/off the auto-brightness adjustment and gesture detection.
    #[deku(id = "0x20")]
    Sensor { en: bool },
    /// Turn on/off the gesture detection only
    #[deku(id = "0x21")]
    Gesture { en: bool },
    /// Turn on/off the auto-brightness adjustment only
    #[deku(id = "0x22")]
    Als { en: bool },

    // --- Graphics commands ---
    /// Set the grey level (0 to 15) used to draw the next graphical element
    #[deku(id = "0x30")]
    Color { color: u8 },
    /// Set a pixel on at the corresponding coordinates
    #[deku(id = "0x31")]
    Point { coord: Point },
    /// Draw a line at the corresponding coordinates
    #[deku(id = "0x32")]
    Line { from: Point, to: Point },
    /// Draw an empty rectangle at the corresponding coordinates
    #[deku(id = "0x33")]
    Rect { from: Point, to: Point },
    /// Draw a full rectangle at the corresponding coordinates
    #[deku(id = "0x34")]
    RectFull { from: Point, to: Point },
    /// Draw an empty circle at the corresponding coordinates
    #[deku(id = "0x35")]
    Circ { center: Point, r: u8 },
    /// Draw a full circle at the corresponding coordinates
    #[deku(id = "0x36")]
    CircFull { center: Point, r: u8 },
    /// Write text `string` at coordinates (x, y) with rotation, font size and color
    #[deku(id = "0x37")]
    Txt {
        pos: Point,
        rotation: u8,
        font_size: u8,
        color: u8,
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, TEXT_LEN)",
            writer = "write_fixed_size_cstr(string, deku::output, TEXT_LEN)"
        )]
        string: String,
    },
    /// Draw multiple connected lines at the corresponding coordinates.
    /// Size: 3 + (n+1) * 4
    #[deku(id = "0x38")]
    Polyline {
        thickness: u8,
        _reserved: u16,
        #[deku(read_all)]
        points: Vec<Point>,
    },
    /// Hold or flush the graphic engine.
    /// When held, new display commands are stored in memory and are displayed when the graphic engine is flushed.
    /// This allows stacking multiple graphic operations and displaying them simultaneously without screen flickering.
    /// The command is nested, the flush must be used the same number of times the hold was used
    /// action = 0 : Hold display
    /// action = 1 : Flush display
    /// action = 0xFF : Reset and flush all stacked hold. To be used when the state of the device is unknown
    /// After a BLE disconnect or an overflow error graphic engine is reset and flushed
    #[deku(id = "0x39")]
    HoldFlush { action: HoldFlushAction },
    /// Draw an arc circle at the corresponding coordinates.
    /// Angles are in degrees, begin at 3 o'clock, and increase clockwise.
    #[deku(id = "0x3C")]
    Arc {
        center: Point,
        r: u8,
        angle_start: i16,
        angle_end: i16,
        thickness: u8,
    },

    // --- Image commands ---
    /// Save an image of `size` bytes and `width` pixels.
    /// Save image according to `format`:
    /// - 0x00: 4bpp
    /// - 0x01: 1bpp, transformed into 4bpp by the firmware before saving
    /// - 0x02: 4bpp with Heatshrink compression, decompressed into 4bpp by the firmware before saving
    /// - 0x03: 4bpp with Heatshrink compression, stored compressed, decompressed into 4bpp before display
    /// - 0x08: 8bpp with 4 bits for grey level and 4 bits for alpha channel
    #[deku(id = "0x41")]
    ImgSave {
        id: u8,
        size: u32,
        width: u16,
        format: u8,
    },
    /// Display image `id` to the corresponding coordinates.
    /// Coordinates are signed, they can be negative.
    #[deku(id = "0x42")]
    ImgDisplay { id: u8, coord: Point },
    /// Stream an image on display without saving it in memory.
    /// Supported formats:
    /// - 0x01: 1bpp
    /// - 0x02: 4bpp with Heatshrink compression
    #[deku(id = "0x44")]
    ImgStream {
        size: u32,
        width: u16,
        coord: Point,
        format: u8,
    },
    /// Delete image.
    /// If `id` = 0xFF, delete all images.
    #[deku(id = "0x46")]
    ImgDelete { id: u8 },
    /// Give the list of saved images.
    #[deku(id = "0x47")]
    ImgList,

    // --- Fonts commands ---
    /// Give the list of saved fonts with their height
    #[deku(id = "0x50")]
    FontList,
    /// Save font `id` of `size` bytes
    ///#[deku(id = "0x51")]
    ///Complicated non-regular use, need special treatment.

    /// Select font which will be used for following text commands
    #[deku(id = "0x52")]
    FontSelect { id: u8 },
    /// Delete font from memory. If `id` = 0xFF, delete all fonts.
    #[deku(id = "0x53")]
    FontDelete { id: u8 },

    // --- Layout commands ---
    /// Save a layout.
    #[deku(id = "0x60")]
    LayoutSave {
        /// Layout number
        id: u8,
        params: LayoutParameters,
    },
    /// Delete a layout. If `id` = 0xFF, delete all layouts.
    #[deku(id = "0x61")]
    LayoutDelete { id: u8 },
    /// Display `text` with layout `id` parameters.
    #[deku(id = "0x62")]
    LayoutDisplay {
        id: u8,
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, TEXT_LEN)",
            writer = "write_fixed_size_cstr(text, deku::output, TEXT_LEN)"
        )]
        text: String,
    },
    /// Clear screen of the corresponding layout area
    #[deku(id = "0x63")]
    LayoutClear { id: u8 },
    /// Give the list of saved layouts
    #[deku(id = "0x64")]
    LayoutList,
    /// Redefine the position of a layout.
    /// The position is saved.
    #[deku(id = "0x65")]
    LayoutPosition { id: u8, pos: LayoutPosition },
    /// Display `text` with layout `id` at the given position.
    /// The position is not saved.
    #[deku(id = "0x66")]
    LayoutDisplayExtended {
        id: u8,
        pos: LayoutPosition,
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, TEXT_LEN)",
            writer = "write_fixed_size_cstr(text, deku::output, TEXT_LEN)"
        )]
        text: String,
        /// Extra commands with the same format as [Commands::LayoutSave]
        #[deku(read_all)]
        extra_cmd: Vec<u8>,
    },
    /// Get a layout parameters
    #[deku(id = "0x67")]
    LayoutGet { id: u8 },
    /// Clear screen of the corresponding layout area
    #[deku(id = "0x68")]
    LayoutClearExtended { id: u8, pos: LayoutPosition },
    /// Clear area and display `text` with layout `id` parameters
    #[deku(id = "0x69")]
    LayoutClearAndDisplay {
        id: u8,
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, TEXT_LEN)",
            writer = "write_fixed_size_cstr(text, deku::output, TEXT_LEN)"
        )]
        text: String,
    },
    /// Clear area and display `text` with layout `id` parameters at given position
    #[deku(id = "0x6A")]
    LayoutClearAndDisplayExtended {
        id: u8,
        pos: LayoutPosition,
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, TEXT_LEN)",
            writer = "write_fixed_size_cstr(text, deku::output, TEXT_LEN)"
        )]
        text: String,
        /// Extra commands with the same format as [Commands::LayoutSave]
        #[deku(read_all)]
        extra_cmd: Vec<u8>,
    },

    // --- Gauge commands ---
    /// Display value (in percentage) of the gauge
    #[deku(id = "0x70")]
    GaugeDisplay { id: u8, value: u8 },
    /// Save the parameters for gauge `id`
    #[deku(id = "0x71")]
    GaugeSave {
        id: u8,
        pos: Point,
        radius: u16,
        inner: u16,
        start: u8,
        end: u8,
        clockwise: u8,
    },
    /// Delete a gauge. if `id` = [ALL], delete all gauges
    #[deku(id = "0x72")]
    GaugeDelete { id: u8 },
    /// Give the list of saved gauges
    #[deku(id = "0x73")]
    GaugeList,
    /// Get a gauge parameters
    #[deku(id = "0x74")]
    GaugeGet { id: u8 },

    // --- Page commands ---
    /// Save a page of layouts
    /// TODO
    #[deku(id = 0x80)]
    PageSave,
    /// Get a page
    #[deku(id = 0x81)]
    PageGet { id: u8 },
    /// Delete a page. If `id` = 0xFF, delete all pages.
    #[deku(id = 0x82)]
    PageDelete { id: u8 },
    /// Display a page, each string are NUL separated
    /// TODO
    #[deku(id = 0x83)]
    PageDisplay { id: u8 },
    /// Clear screen of the corresponding page area
    #[deku(id = 0x84)]
    PageClear { id: u8 },
    /// List pages in memory
    #[deku(id = 0x85)]
    PageList,
    /// Clear area and display a page, each string are NUL separated
    /// TODO
    #[deku(id = 0x86)]
    PageClearAndDisplay { id: u8 },

    // --- Animation commands ---
    /// save an animation
    #[deku(id = "0x95")]
    AnimSave {
        id: u8,
        /// Total animation size, in bytes
        total_size: u32,
        /// Reference frame size in bytes
        img_size: u32,
        /// Reference image width in pixel
        width: u16,
        /// format of reference frame
        /// 0x00: 4bpp
        /// 0x02: 4bpp with HeatShrink compression, decompressed to 4bpp by the firmware before
        /// saving
        fmt: u8,
        /// Reference frame size before it is decompressed. for 4bpp it's equal to img_size
        img_compressed_size: u32,
    },
    /// Delete an animation. If `id` = 0xFF, delete all animations
    #[deku(id = "0x96")]
    AnimDelete { id: u8 },
    /// Display animation `id` to the corresponding coordinates.
    #[deku(id = "0x97")]
    AnimDisplay {
        /// Value specified by the user, used to stop the animation later
        handler_id: u8,
        /// Animation `id`
        id: u8,
        /// Set the inter-frame duration in ms
        delay: u16,
        /// Repeat count, or 0xFF for infinite repetition
        repeat: u8,
        pos: Point,
    },
    /// Stop and clear the screen of the corresponding animation.
    /// If `handler_id` = 0xFF, clear all animations.
    #[deku(id = "0x98")]
    AnimClear { handler_id: u8 },
    /// Get list of saved animations
    #[deku(id = "0x99")]
    AnimList,

    // --- Statistics commands ---
    /// Get the number of pixels activated on the display
    #[deku(id = "0xA5")]
    PixelCount,

    // --- Configuration commands ---
    /// Write configuration. Configurations are associated with layouts, images, etc.
    /// **Warning** This command is allowed only if the battery is above 5%
    #[deku(id = "0xD0")]
    CfgWrite {
        /// Name of the configuration
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, NAME_LEN)",
            writer = "write_fixed_size_cstr(name, deku::output, NAME_LEN)"
        )]
        name: String,
        /// Provided by the user for tracking versions
        version: u32,
        /// If the configuration already exists, the same password must be provided as the one
        /// during the creation.
        password: u32,
    },
    /// Get the number of elements stored in the configuration
    #[deku(id = "0xD1")]
    CfgRead {
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, NAME_LEN)",
            writer = "write_fixed_size_cstr(name, deku::output, NAME_LEN)"
        )]
        name: String,
    },
    /// Select the current configuration used to display layouts, images, etc.
    #[deku(id = "0xD2")]
    CfgSet {
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, NAME_LEN)",
            writer = "write_fixed_size_cstr(name, deku::output, NAME_LEN)"
        )]
        name: String,
    },
    #[deku(id = "0xD3")]
    CfgList,
    /// Rename a configuration
    #[deku(id = "0xD4")]
    CfgRename {
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, NAME_LEN)",
            writer = "write_fixed_size_cstr(old, deku::output, NAME_LEN)"
        )]
        old: String,
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, NAME_LEN)",
            writer = "write_fixed_size_cstr(new, deku::output, NAME_LEN)"
        )]
        new: String,
        password: u32,
    },
    /// Delete a configuration and all elements associated
    #[deku(id = "0xD5")]
    CfgDelete {
        #[deku(
            reader = "read_fixed_size_cstr(deku::reader, NAME_LEN)",
            writer = "write_fixed_size_cstr(name, deku::output, NAME_LEN)"
        )]
        name: String,
    },
    /// Delete the configuration that has not been used for the longest time
    #[deku(id = "0xD6")]
    CfgDeleteLessUsed,
    /// Get free space available to store layouts, images, etc
    #[deku(id = "0xD7")]
    CfgFreeSpace,
    /// Get the number of configurations in memory
    #[deku(id = "0xD8")]
    CfgGetNb,

    // --- Device commands ---
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

// Ttrait implementations
impl Serializable for Command {
    type Item = Self;

    /// Access the discriminant as unique ID
    fn id(&self) -> Result<u8, DekuError> {
        self.deku_id()
    }

    /// Access data bytes for serialization.
    /// This might become expensive but we'll deal with that later.
    fn data_bytes(&self) -> Result<Vec<u8>, DekuError> {
        let mut bytes: Vec<u8> = self.to_bytes()?;
        bytes.remove(0);
        Ok(bytes)
    }

    /// Extract CommandID and data bytes from Command
    fn as_bytes(&self) -> Result<(u8, Vec<u8>), DekuError> {
        let data = self.data_bytes()?;
        Ok((self.id()?, data))
    }
}

impl Deserializable for Command {
    type Item = Self;

    /// Create a Command from the CommandID and data.
    fn from_data(id: u8, data: Option<&[u8]>) -> Result<Self, DekuError> {
        let mut bytes = vec![id];
        if let Some(data) = data {
            bytes.extend_from_slice(&data);
        }
        let (_rest, cmd) = Command::from_bytes((&bytes, 0))?;
        Ok(cmd)
    }
}

// ---------------------------------------------------------------------------
// All responses
// ---------------------------------------------------------------------------

/// These map to the responses ActiveLookToMaster
#[derive(Debug, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8")]
#[repr(u8)]
pub enum Response {
    // --- General commands --
    /// Battery level in % (0x64 = 100%)
    #[deku(id = "0x05")]
    Battery { level: u8 },
    /// Firmware version and Serial Number
    #[deku(id = "0x06")]
    Version {
        fw_version: [u8; 4],
        mfc_year: u8,
        mfc_week: u8,
        serial_number: [u8; 3],
    },
    /// Global settings
    #[deku(id = "0x0A")]
    Settings {
        x: i8,
        y: i8,
        luma: u8,
        als_enable: u8,
        gesture_enable: u8,
    },

    // --- Image commands ---
    /// List images in memory. `height` and `width` are in pixels. Listing is not sorted.
    #[deku(id = "0x47")]
    ImgList {
        #[deku(read_all)]
        list: Vec<ImgListItem>,
    },

    // --- Fonts commands ---
    /// List of font in memory, with their height. Listing is not sorted.
    #[deku(id = "0x50")]
    FontList {
        #[deku(read_all)]
        list: Vec<FontItem>,
    },

    // --- Layout commands ---
    /// List of layouts in memory. Listing is not sorted.
    #[deku(id = "0x64")]
    LayoutList {
        #[deku(read_all)]
        list: Vec<u8>,
    },
    /// Layout parameters without `id`
    #[deku(id = "0x67")]
    LayoutGet { params: LayoutParameters },

    // --- Gauge commands ---
    /// List of gauges in memory. Not sorted.
    #[deku(id = "0x73")]
    GaugeList {
        #[deku(read_all)]
        list: Vec<u8>,
    },
    /// Gauge parameters without `id`
    #[deku(id = "0x74")]
    GaugeGet {
        pos: Point,
        radius: u16,
        inner: u16,
        start: u8,
        end: u8,
        clockwise: u8,
    },

    // --- Page commands ---
    /// Page with layout parameters
    #[deku(id = "0x81")]
    PageGet { id: u8 },
    /// List of page IDs in memory. Listing is not sorted
    #[deku(id = 0x85)]
    PageList {
        #[deku(read_all)]
        list: Vec<u8>,
    },

    // --- Animation commands ---
    /// List of animations in memory. Listing is not sorted.
    #[deku(id = "0x99")]
    AnimList {
        #[deku(read_all)]
        list: Vec<u8>,
    },

    // --- Statistics commands ---
    /// Number of pixels activated on the display
    #[deku(id = "0xA5")]
    PixelCount { count: u32 },

    // --- Configuration commands ---
    /// Number of elements stored in the configuration
    #[deku(id = "0xD2")]
    CfgRead {
        version: u32,
        nb_img: u8,
        nb_layout: u8,
        nb_font: u8,
        nb_page: u8,
        nb_gauge: u8,
    },
    #[deku(id = "0xD3")]
    CfgList {
        #[deku(read_all)]
        list: Vec<CfgItem>,
    },
    #[deku(id = "0xD7")]
    CfgFreeSpace {
        /// Total size available in bytes
        total_size: u32,
        /// Free space available in bytes
        free_space: u32,
    },
    /// Number of configurations stored in memory
    #[deku(id = "0xD8")]
    CfgGetNb { nb_config: u8 },

    // --- Device commands ---
    /// This message is sent asynchronously when there is an error during command processing.
    /// `cmd_id` is the ID of the command who got an error.
    #[deku(id = "0xE2")]
    CmdError {
        cmd_id: u8,
        error: CmdError,
        sub_error: u8,
    },
    ///
    #[deku(id = "0xE3")]
    RdDevInfo {
        #[deku(read_all)]
        parameters: Vec<u8>,
    },
}

// Ttrait implementations
impl Serializable for Response {
    type Item = Self;

    /// Access the discriminant as unique ID
    fn id(&self) -> Result<u8, DekuError> {
        self.deku_id()
    }

    /// Access data bytes for serialization.
    /// This might become expensive but we'll deal with that later.
    fn data_bytes(&self) -> Result<Vec<u8>, DekuError> {
        let mut bytes: Vec<u8> = self.to_bytes()?;
        bytes.remove(0);
        Ok(bytes)
    }

    /// Extract CommandID and data bytes from Command
    fn as_bytes(&self) -> Result<(u8, Vec<u8>), DekuError> {
        let data = self.data_bytes()?;
        Ok((self.id()?, data))
    }
}

impl Deserializable for Response {
    type Item = Self;

    /// Create a Command from the CommandID and data.
    fn from_data(id: u8, data: Option<&[u8]>) -> Result<Self, DekuError> {
        let mut bytes = vec![id];
        if let Some(data) = data {
            bytes.extend_from_slice(&data);
        }
        let (_rest, cmd) = Self::from_bytes((&bytes, 0))?;
        Ok(cmd)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
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
    fn test_simple_serialization() {
        // Serialization
        let expected: &[u8] = &[0x00, 0x01];
        let cmd = Command::PowerDisplay { en: true as u8 };
        let bytes = cmd.to_bytes().unwrap();
        assert_eq!(expected, bytes);

        let data = cmd.data_bytes().unwrap();
        assert_eq!(expected[1..], data);

        // Deserialization
        let other = Command::from_data(0x00, Some(&[0x01])).unwrap();
        assert_eq!(cmd, other);
    }

    #[test]
    fn test_deserialization_no_data() {
        let bytes = [0x01];
        let expected = Command::Clear;

        let cmd = Command::from_data(bytes[0], None).unwrap();
        assert_eq!(expected, cmd);
    }

    #[test]
    fn test_vec_serialization() {
        let bytes: &[u8] = &[1, 2, 3];
        let expected = Response::RdDevInfo {
            parameters: vec![1, 2, 3],
        };
        // Serialization
        let data = expected.data_bytes().unwrap();
        assert_eq!(bytes, data);

        // Deserialization
        let res = Response::from_data(0xE3, Some(&bytes)).unwrap();
        assert_eq!(expected, res);
    }

    #[test]
    fn test_fixed_string_short() {
        let bytes: &[u8] = &[
            42, // id
            0x30, 0x31, 0x32, 0x00, // text
        ];
        let expected = Command::LayoutDisplay {
            id: 42,
            text: String::from("012"),
        };
        let data = expected.data_bytes().unwrap();
        assert_eq!(bytes, data);

        let cmd = Command::from_data(0x62, Some(bytes)).unwrap();
        assert_eq!(expected, cmd);

        // how to access the returned value
        match cmd {
            Command::LayoutDisplay { id, text } => assert_eq!(text, "012"),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_fixed_string_exact() {
        let bytes: &[u8] = &[0x30; TEXT_LEN + 1];
        let expected = Command::LayoutDisplay {
            id: 0x30,
            text: String::from_utf8(vec![0x30; TEXT_LEN]).unwrap(),
        };
        let data = expected.data_bytes().unwrap();
        assert_eq!(bytes, data);

        let cmd = Command::from_data(0x62, Some(bytes)).unwrap();
        assert_eq!(expected, cmd);
    }
}
