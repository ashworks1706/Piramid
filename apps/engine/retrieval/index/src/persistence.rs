//! Index sidecar persistence.
//!
//! An index owns its own on-disk format, so save/load lives here rather than in `storage/`.

use crate::{SerializableIndex, VectorIndex};
use piramid_core::error::Result;
use std::fs;
use std::path::Path;

/// Sidecar path for a collection's index.
pub fn get_index_file_path(collection_path: &str) -> String {
    format!("{}.vecindex.db", collection_path)
}

/// Serialize `index` to its sidecar.
pub fn save_vector_index(collection_path: &str, index: &dyn VectorIndex) -> Result<()> {
    let serializable = index.to_serializable();

    let bytes = bincode::serialize(&serializable)?;
    let index_path = get_index_file_path(collection_path);
    fs::write(index_path, bytes)?;
    Ok(())
}

/// Load a previously saved index, if one exists.
pub fn load_vector_index(collection_path: &str) -> Result<Option<Box<dyn VectorIndex>>> {
    // construct the expected file path for the index based on the collection path.
    let index_path = get_index_file_path(collection_path);

    if !Path::new(&index_path).exists() {
        return Ok(None);
    }

    let bytes = fs::read(index_path)?;
    let serializable: SerializableIndex = bincode::deserialize(&bytes)?;
    Ok(Some(serializable.to_trait_object()))
}
