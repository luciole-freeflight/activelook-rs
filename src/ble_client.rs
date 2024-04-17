use deku::prelude::*;
use thiserror::Error;

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

#[deku_derive(DekuRead, DekuWrite)]
#[derive(Debug, Eq, PartialEq)]
pub struct Header {
    cmd_id: u8,
    //cmd_format: CommandFormat, // Only used for de/serialization - ignore
    length: u16,
    query_id: Vec<u8>,
}

