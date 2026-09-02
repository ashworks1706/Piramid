//! The offset index: document id to byte range in the record file.

use serde::{Deserialize, Serialize};

/// Where one document's bytes live in the record file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPointer {
    /// Byte offset from the start of the record file.
    pub offset: u64,
    /// Length of the serialized document in bytes.
    pub length: u32,
}

impl EntryPointer {
    /// Point at `length` bytes starting at `offset`.
    pub fn new(offset: u64, length: u32) -> Self {
        Self { offset, length }
    }
}
