//! Traits used in the crate
use deku::prelude::*;

/// Serialize to a bytestream
pub trait Serializable {
    type Item;

    fn id(&self) -> Result<u8, DekuError>;
    fn data_bytes(&self) -> Result<Vec<u8>, DekuError>;
    fn as_bytes(&self) -> Result<(u8, Vec<u8>), DekuError>;
}

/// Deserialize from a bytestream
pub trait Deserializable {
    type Item;

    fn from_data(id: u8, data: Option<&[u8]>) -> Result<Self::Item, DekuError>;
}
