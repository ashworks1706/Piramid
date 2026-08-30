//! Index sidecar persistence: an index owns its own on-disk format.

use crate::{SerializableIndex, VectorIndex};
use piramid_core::error::Result;
use std::fs;
use std::path::Path;

use piramid_storage::persistence::SidecarManager;

/// Sidecar path for a collection's index, owned by [`SidecarManager`].
fn get_index_file_path(collection_path: &str) -> String {
    SidecarManager::at(collection_path).vector_index_path()
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
    let index_path = get_index_file_path(collection_path);

    if !Path::new(&index_path).exists() {
        return Ok(None);
    }

    let bytes = fs::read(index_path)?;
    let serializable: SerializableIndex = bincode::deserialize(&bytes)?;
    Ok(Some(serializable.to_trait_object()))
}
