//! The offset index: document id to byte range in the record file.
//!
//! Named `offsets`, not `index` — this maps a `Uuid` to where its bytes live, while
//! `piramid-index` decides which vectors are worth reading.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use piramid_core::error::{Result, StorageError};

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

/// Persist the offset index sidecar.
pub fn save_index(path: &str, index: &HashMap<Uuid, EntryPointer>) -> Result<()> {
    let index_path = format!("{}.index.db", path);
    let index_data = bincode::serialize(index)?;
    std::fs::write(index_path, index_data)?;
    Ok(())
}

/// Load the offset index sidecar.
pub fn load_index(path: &str) -> Result<HashMap<Uuid, EntryPointer>> {
    let index_path = format!("{}.index.db", path);

    let mut index_file = match std::fs::File::open(&index_path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(error) => return Err(error.into()),
    };

    use std::io::Read;
    let mut index_data = Vec::new();
    index_file.read_to_end(&mut index_data)?;
    bincode::deserialize(&index_data).map_err(|e| {
        StorageError::CorruptedIndex(format!("failed to decode {index_path}: {e}")).into()
    })
}

/// Path of the write-ahead log beside a record file.
pub fn get_wal_path(storage_path: &str) -> String {
    format!("{}.wal.db", storage_path)
}
