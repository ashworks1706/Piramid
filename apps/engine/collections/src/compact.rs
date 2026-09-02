//! Compaction: rewrite the record store without dead entries.

use std::collections::HashMap;

use super::collection::Collection;
use piramid_core::error::Result;
use piramid_index::save_vector_index;
use piramid_index::HashMapVectorReader;
use piramid_storage::document::Document;
use piramid_storage::record_store::RecordStore;
use piramid_storage::SidecarManager;

/// Compact a collection by rewriting live documents into a fresh file and rebuilding indexes.
pub fn compact(storage: &mut Collection) -> Result<CompactStats> {
    let original_entries = storage.index.len();
    let docs: Vec<Document> = storage.get_all()?;

    let temp_path = SidecarManager::at(&storage.path).compact_path();
    match std::fs::remove_file(&temp_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let mut temp_store = RecordStore::open(&temp_path, &storage.config, &HashMap::new())?;
    let mut new_index = HashMap::with_capacity(docs.len());
    let mut new_vectors = HashMap::with_capacity(docs.len());
    let mut new_vector_index =
        piramid_index::create_index(&storage.config.index, storage.config.execution, docs.len());
    let mut new_metadata = storage.metadata.clone();
    new_metadata.update_vector_count(0);

    for doc in docs {
        let id = doc.id;
        let vector = doc.vector().to_vec();
        let bytes = RecordStore::encode_document(&doc)?;
        let pointer = temp_store.append(&bytes)?;
        new_metadata.set_dimensions(vector.len())?;
        new_index.insert(id, pointer);
        new_vectors.insert(id, vector.clone());
        let reader = HashMapVectorReader::new(&new_vectors);
        new_vector_index.insert(id, &vector, &reader)?;
    }
    new_metadata.update_vector_count(new_index.len());

    temp_store.sync()?;
    drop(temp_store);
    std::fs::rename(&temp_path, &storage.path)?;

    storage.record_store = RecordStore::open(&storage.path, &storage.config, &new_index)?;
    storage.index = new_index;
    storage.vector_index = new_vector_index;
    storage.metadata = new_metadata;
    storage.clear_caches_for_rebuild();
    storage.rebuild_vector_cache()?;

    let sidecars = SidecarManager::at(&storage.path);
    sidecars.save_offsets(&storage.index)?;
    save_vector_index(&storage.path, storage.vector_index())?;
    sidecars.save_manifest(&storage.metadata)?;
    // Sidecars are durable before we drop the WAL entries they made redundant.
    storage.checkpoint.wal.rotate()?;

    Ok(CompactStats {
        original_entries,
        compacted_entries: storage.index.len(),
    })
}

#[derive(Debug)]
pub struct CompactStats {
    pub original_entries: usize,
    pub compacted_entries: usize,
}
