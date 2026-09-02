//! Checkpoint bookkeeping: when to flush sidecars, and clearing the WAL once they are durable.

use super::collection::Collection;
use piramid_core::error::Result;
use piramid_database::storage::wal::Wal;
use piramid_database::storage::SidecarManager;
use piramid_retrieval::index::save_vector_index as save_vec_idx;

pub struct CheckpointManager {
    pub wal: Wal,
    operation_count: usize,
    last_checkpoint_ts: Option<u64>,
}

impl CheckpointManager {
    pub fn new(wal: Wal) -> Self {
        Self {
            wal,
            operation_count: 0,
            last_checkpoint_ts: None,
        }
    }

    pub fn should_checkpoint(&mut self, cfg: &piramid_core::config::WalConfig) -> bool {
        if !cfg.enabled {
            return false;
        }
        self.operation_count += 1;
        self.operation_count >= cfg.checkpoint_frequency
    }

    pub fn reset_counter(&mut self) {
        self.operation_count = 0;
    }

    pub fn record_checkpoint(&mut self, ts: u64) {
        self.last_checkpoint_ts = Some(ts);
    }

    pub fn last_checkpoint(&self) -> Option<u64> {
        self.last_checkpoint_ts
    }
}

pub fn save_index(collection: &Collection) -> Result<()> {
    SidecarManager::at(&collection.path).save_offsets(&collection.index)
}

pub fn save_vector_index(collection: &Collection) -> Result<()> {
    save_vec_idx(&collection.path, collection.vector_index.as_ref())
}

pub fn save_manifest(collection: &Collection) -> Result<()> {
    SidecarManager::at(&collection.path).save_manifest(&collection.manifest)
}

pub fn checkpoint(collection: &mut Collection) -> Result<()> {
    let timestamp = piramid_core::clock::unix_secs();

    // All three sidecars land before the WAL is cleared below, so a crash mid-checkpoint replays
    // rather than loses.
    save_index(collection)?;
    save_vector_index(collection)?;
    save_manifest(collection)?;

    if collection.config.wal.enabled {
        collection.checkpoint.wal.checkpoint(timestamp)?;
        collection.checkpoint.record_checkpoint(timestamp);
        let last_seq = collection.checkpoint.wal.next_seq.saturating_sub(1);
        SidecarManager::at(&collection.path).save_wal_meta(last_seq)?;
        collection.checkpoint.wal.rotate()?;
    }

    Ok(())
}

pub fn flush(collection: &mut Collection) -> Result<()> {
    collection.checkpoint.wal.flush()?;
    Ok(())
}
