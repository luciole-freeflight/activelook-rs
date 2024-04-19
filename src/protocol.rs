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
use crate::commands::{self, Command, Deserializable, Response, Serializable};

// ---------------------------------------------------------------------------
//pub fn encode(cmd: &Command)
