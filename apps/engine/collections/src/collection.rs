use std::collections::HashMap;
use uuid::Uuid;

use super::checkpoint::CheckpointManager;
use crate::cache::CacheManager;
use piramid_core::error::Result;
use piramid_database::storage::manifest::CollectionMetadata;
use piramid_database::storage::persistence::{warm_file, EntryPointer};
use piramid_database::storage::record_store::RecordStore;
use piramid_database::storage::SidecarManager;
use piramid_retrieval::index::save_vector_index;
use piramid_retrieval::index::{HashMapVectorReader, VectorIndex, VectorReader};

pub struct Collection {
    pub(super) record_store: RecordStore,
    pub(super) index: HashMap<Uuid, EntryPointer>,
    pub(super) vector_index: Box<dyn VectorIndex>,
    pub(super) cache: CacheManager,
    pub config: piramid_core::config::CollectionConfig,
    pub metadata: CollectionMetadata,
    pub path: String,
    pub checkpoint: CheckpointManager,
}

impl Collection {
    pub(super) fn track_operation(&mut self) -> Result<()> {
        let interval_due = if let Some(last) = self.checkpoint.last_checkpoint() {
            if let Some(interval) = self.config.wal.checkpoint_interval_secs {
                let now = piramid_core::clock::unix_secs();
                now.saturating_sub(last) >= interval
            } else {
                false
            }
        } else {
            false
        };

        if self.checkpoint.should_checkpoint(&self.config.wal) || interval_due {
            super::checkpoint::checkpoint(self)?;
            self.checkpoint.reset_counter();
        }
        Ok(())
    }

    pub fn metadata(&self) -> &CollectionMetadata {
        &self.metadata
    }

    pub fn count(&self) -> usize {
        self.index.len()
    }

    /// Approximate resident size: mmap + offset index + caches + ANN structure.
    pub fn memory_usage_bytes(&self) -> Result<usize> {
        let index_size = self.index.capacity() * std::mem::size_of::<(Uuid, EntryPointer)>();

        Ok(self.record_store.mapped_len()?
            + index_size
            + self.cache.memory_usage_bytes()
            + self.vector_index.stats().memory_usage_bytes)
    }

    pub fn vector_index(&self) -> &dyn VectorIndex {
        self.vector_index.as_ref()
    }

    pub fn cache_usage_bytes(&self) -> usize {
        self.cache.memory_usage_bytes()
    }

    pub fn metadata_cache_usage_bytes(&self) -> usize {
        self.cache.metadata_usage_bytes()
    }

    pub fn clear_metadata_cache(&mut self) -> usize {
        self.cache.clear_metadata()
    }

    pub fn clear_caches_for_rebuild(&mut self) {
        self.cache.clear_all();
    }

    /// Faults frequently used files into the page cache to reduce cold-start latency.
    pub fn warm_page_cache(&self) {
        self.record_store.warm_page_cache();
        let sidecars = SidecarManager::at(&self.path);
        for path in [
            sidecars.vector_index_path(),
            sidecars.offsets_path(),
            sidecars.wal_path(),
        ] {
            if let Err(error) = warm_file(&path) {
                tracing::warn!(
                    target: "piramid::collections",
                    %path,
                    %error,
                    "could not warm page cache for file"
                );
            }
        }
    }

    pub fn vectors_view(&self) -> &HashMap<Uuid, Vec<f32>> {
        self.cache.vectors()
    }

    pub fn vector_reader(&self) -> &dyn VectorReader {
        &self.cache
    }

    pub fn metadata_view(&self) -> &HashMap<Uuid, piramid_core::metadata::Metadata> {
        self.cache.metadata()
    }

    pub fn config(&self) -> &piramid_core::config::CollectionConfig {
        &self.config
    }

    pub fn get_all(&self) -> Result<Vec<piramid_database::storage::document::Document>> {
        let mut all_entries = Vec::new();
        for id in self.index.keys() {
            if let Some(entry) = super::operations::get(self, id)? {
                all_entries.push(entry);
            }
        }
        Ok(all_entries)
    }

    pub(super) fn rebuild_vector_cache(&mut self) -> Result<()> {
        let mut cache = CacheManager::new(self.config.cache);
        for id in self.index.keys() {
            if let Some(entry) = super::operations::get(self, id)? {
                cache.put_vector(*id, entry.vector().to_vec());
                cache.put_metadata(*id, entry.metadata.clone());
            }
        }
        self.cache = cache;
        Ok(())
    }

    /// Rebuild the vector index from on-disk data and persist it.
    pub fn rebuild_index(&mut self) -> Result<()> {
        let mut vectors: HashMap<Uuid, Vec<f32>> = HashMap::new();

        for (id, pointer) in &self.index {
            let entry = self.record_store.read_document(pointer)?;
            vectors.insert(*id, entry.vector().to_vec());
        }

        let mut new_index = piramid_retrieval::index::create_index(
            &self.config.index,
            self.config.execution,
            self.index.len(),
        );
        let reader = HashMapVectorReader::new(&vectors);
        for (id, vec) in &vectors {
            new_index.insert(*id, vec, &reader)?;
        }

        self.vector_index = new_index;
        self.rebuild_vector_cache()?;
        save_vector_index(self.path.as_str(), self.vector_index())?;
        Ok(())
    }
}
