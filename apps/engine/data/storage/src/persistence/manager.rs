//! One owner for every sidecar path and format beside a collection's record file.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::offsets::EntryPointer;
use crate::manifest::{CollectionMetadata, SCHEMA_VERSION};
use piramid_core::error::{Result, StorageError};

/// The sidecar domain entry for one collection.
///
/// Every file that sits beside `{base}` — offsets, manifest, WAL, WAL meta, vector index — gets
/// its path and its serialization from here, so path knowledge lives in exactly one place. A new
/// sidecar means a new method on this type, not a `format!` at a call site.
#[derive(Clone, Copy)]
pub struct SidecarManager<'a> {
    base: &'a str,
}

/// Checkpoint bookkeeping persisted beside the WAL.
#[derive(Serialize, Deserialize, Default)]
struct WalMeta {
    last_checkpoint_seq: u64,
}

impl<'a> SidecarManager<'a> {
    /// The sidecars beside the record file at `base`.
    pub fn at(base: &'a str) -> Self {
        Self { base }
    }

    /// Path of the write-ahead log.
    pub fn wal_path(&self) -> String {
        format!("{}.wal.db", self.base)
    }

    /// Path of the WAL checkpoint bookkeeping file.
    pub fn wal_meta_path(&self) -> String {
        format!("{}.wal.meta", self.base)
    }

    /// Path of the offset-index sidecar.
    pub fn offsets_path(&self) -> String {
        format!("{}.index.db", self.base)
    }

    /// Path of the collection manifest sidecar.
    pub fn manifest_path(&self) -> String {
        format!("{}.metadata.db", self.base)
    }

    /// Path of the ANN index sidecar. `piramid-index` owns its format; this owns its place.
    pub fn vector_index_path(&self) -> String {
        format!("{}.vecindex.db", self.base)
    }

    /// Persist the offset index.
    pub fn save_offsets(&self, index: &HashMap<Uuid, EntryPointer>) -> Result<()> {
        fs::write(self.offsets_path(), bincode::serialize(index)?)?;
        Ok(())
    }

    /// Load the offset index; a missing sidecar is an empty collection.
    pub fn load_offsets(&self) -> Result<HashMap<Uuid, EntryPointer>> {
        let path = self.offsets_path();
        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
            Err(error) => return Err(error.into()),
        };
        bincode::deserialize(&data).map_err(|e| {
            StorageError::CorruptedIndex(format!("failed to decode {path}: {e}")).into()
        })
    }

    /// Persist the collection manifest.
    pub fn save_manifest(&self, metadata: &CollectionMetadata) -> Result<()> {
        fs::write(self.manifest_path(), bincode::serialize(metadata)?)?;
        Ok(())
    }

    /// Load the manifest, refusing one written under a different schema version.
    pub fn load_manifest(&self) -> Result<Option<CollectionMetadata>> {
        let path = self.manifest_path();
        if !Path::new(&path).exists() {
            return Ok(None);
        }

        let bytes = fs::read(&path)?;
        let metadata: CollectionMetadata = bincode::deserialize(&bytes)
            .map_err(|e| StorageError::CorruptedData(format!("failed to read manifest: {e}")))?;
        if metadata.schema_version != SCHEMA_VERSION {
            return Err(StorageError::CorruptedData(format!(
                "Schema version mismatch: expected {}, found {}",
                SCHEMA_VERSION, metadata.schema_version
            ))
            .into());
        }
        Ok(Some(metadata))
    }

    /// Record the last checkpointed WAL sequence, atomically via a temp file.
    pub fn save_wal_meta(&self, last_checkpoint_seq: u64) -> Result<()> {
        let meta_path = self.wal_meta_path();
        let tmp_path = format!("{meta_path}.tmp");
        let meta = WalMeta {
            last_checkpoint_seq,
        };
        fs::write(&tmp_path, serde_json::to_vec(&meta)?)?;
        fs::rename(&tmp_path, &meta_path)?;
        let file = fs::File::open(&meta_path)?;
        file.sync_all()?;
        Ok(())
    }

    /// Last checkpointed WAL sequence; 0 when no checkpoint has ever happened.
    pub fn load_wal_meta(&self) -> Result<u64> {
        let data = match fs::read(self.wal_meta_path()) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(error) => return Err(error.into()),
        };
        let meta: WalMeta = serde_json::from_slice(&data)?;
        Ok(meta.last_checkpoint_seq)
    }
}
