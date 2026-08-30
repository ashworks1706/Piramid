//! Opening a collection: load sidecars, replay the WAL, rebuild what is missing.
use std::collections::HashMap;
use uuid::Uuid;

use super::checkpoint::CheckpointManager;
use super::collection::Collection;
use super::CollectionOpenOptions;
use piramid_cache::CacheManager;
use piramid_core::error::{Result, StorageError};
use piramid_index::load_vector_index;
use piramid_index::HashMapVectorReader;
use piramid_storage::document::Document;
use piramid_storage::manifest::CollectionMetadata;
use piramid_storage::persistence::SidecarManager;
use piramid_storage::record_store::RecordStore;
use piramid_storage::wal::{Wal, WalEntry};

pub struct CollectionBuilder;

impl CollectionBuilder {
    pub fn open(path: &str, options: CollectionOpenOptions) -> Result<Collection> {
        let config = options.config;

        Collection::init_rayon_pool(&config.parallelism);

        // Naming it "unknown" would put a collection on disk under a name that matches nothing
        // the caller asked for, and every later lookup by name would miss.
        let collection_name = std::path::Path::new(path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| {
                StorageError::InvalidVectorData(format!(
                    "collection path '{path}' has no usable file stem to name the collection"
                ))
            })?
            .to_string();

        let sidecars = SidecarManager::at(path);
        let index = sidecars.load_offsets()?;
        let record_store = RecordStore::open(path, &config, &index)?;

        let metadata = match sidecars.load_manifest()? {
            Some(meta) => {
                let mut meta = meta;
                meta.update_vector_count(index.len());
                meta
            }
            None => CollectionMetadata::new(collection_name),
        };

        let loaded_vector_index = load_vector_index(path)?;
        let vector_index_missing = loaded_vector_index.is_none();
        let mut vector_index = match loaded_vector_index {
            Some(loaded_index) => loaded_index,
            None => piramid_index::create_index(&config.index, index.len()),
        };

        let min_seq = if config.wal.enabled {
            sidecars.load_wal_meta()?
        } else {
            0
        };
        let next_seq = min_seq + 1;

        let wal_path = sidecars.wal_path();

        let wal = if config.wal.enabled {
            Wal::new(wal_path.into(), next_seq)?
        } else {
            Wal::disabled(wal_path.into(), next_seq)?
        };

        let checkpoint = CheckpointManager::new(wal);

        let wal_entries = if config.wal.enabled {
            checkpoint.wal.replay(min_seq)?
        } else {
            Vec::new()
        };

        // Replay into a temporary collection: the WAL is only safe to clear once the replayed
        // state is durable, so the checkpoint has to succeed first.

        if !wal_entries.is_empty() {
            let mut recovered_collection = Collection {
                record_store,
                index,
                vector_index,
                cache: CacheManager::new(config.cache),
                config: config.clone(),
                metadata,
                path: path.to_string(),
                checkpoint,
            };

            Self::replay_wal(&mut recovered_collection, wal_entries)?;

            recovered_collection.rebuild_vector_cache()?;

            super::checkpoint::checkpoint(&mut recovered_collection)?;

            return Ok(recovered_collection);
        }

        // Records exist but the ANN sidecar does not — rebuild it from the record store.
        if !index.is_empty() && vector_index_missing {
            Self::rebuild_vector_index(&mut vector_index, &index, &record_store)?;
        }

        let mut collection = Collection {
            record_store,
            index,
            vector_index,
            cache: CacheManager::new(config.cache),
            config,
            metadata,
            path: path.to_string(),
            checkpoint,
        };

        collection.rebuild_vector_cache()?;
        Ok(collection)
    }

    fn replay_wal(collection: &mut Collection, entries: Vec<WalEntry>) -> Result<()> {
        for entry in entries {
            match entry {
                // An update is a delete followed by an insert so the ANN index sees the change.
                WalEntry::Insert {
                    id,
                    vector,
                    text,
                    metadata,
                    ..
                } => {
                    let vec_entry = Document {
                        id,
                        vector,
                        text,
                        metadata,
                    };
                    super::operations::insert_internal(collection, vec_entry)?;
                }

                WalEntry::Update {
                    id,
                    vector,
                    text,
                    metadata,
                    ..
                } => {
                    super::operations::delete_internal(collection, &id);
                    let vec_entry = Document {
                        id,
                        vector,
                        text,
                        metadata,
                    };
                    super::operations::insert_internal(collection, vec_entry)?;
                }
                WalEntry::Delete { id, .. } => {
                    super::operations::delete_internal(collection, &id);
                }
                WalEntry::Checkpoint { .. } => {}
            }
        }
        Ok(())
    }

    fn rebuild_vector_index(
        vector_index: &mut Box<dyn piramid_index::VectorIndex>,
        index: &HashMap<Uuid, piramid_storage::persistence::EntryPointer>,
        record_store: &RecordStore,
    ) -> Result<()> {
        // Read every live record through the offset index and re-insert it, which is what makes
        // the ANN index disposable: it can always be reconstructed from the record store.
        let mut vectors: HashMap<Uuid, Vec<f32>> = HashMap::new();
        for (id, idx_entry) in index {
            let entry = record_store.read_document(idx_entry)?;
            vectors.insert(*id, entry.vector().to_vec());
        }

        let reader = HashMapVectorReader::new(&vectors);
        for (id, vector) in &vectors {
            vector_index.insert(*id, vector, &reader)?;
        }
        Ok(())
    }
}
