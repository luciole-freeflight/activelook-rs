//! Traits used in the crate
use deku::prelude::*;

/// Serialize to a bytestream
pub trait Serializable: Clone {
    /// Returns the ID of the [Command] or [Response]
    fn id(&self) -> Result<u8, DekuError>;

    /// Returns the byte representation of the data
    fn data_bytes(&self) -> Result<Vec<u8>, DekuError>;

    /// This returns the ActiveLook command into the u8 ID representing the [Command] or [Response],
    /// as well as the data bytes.
    fn as_bytes(&self) -> Result<(u8, Vec<u8>), DekuError>;

    /// Use this function to split the byte representation into smaller chunks. This is useful to
    /// send bigger images to the ActiveLook glasses.
    fn as_bytes_chunks(&self, chunk_size: usize) -> Result<(u8, Vec<Vec<u8>>), DekuError>;
}

/// Deserialize from a bytestream
pub trait Deserializable {
    type Item;

    fn from_data(id: u8, data: Option<&[u8]>) -> Result<Self::Item, DekuError>;
}
