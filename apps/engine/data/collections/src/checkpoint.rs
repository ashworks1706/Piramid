//! Checkpoint bookkeeping: when to flush sidecars, and clearing the WAL once they are durable.

use super::collection::Collection;
use piramid_core::error::Result;
use piramid_index::save_vector_index as save_vec_idx;
use piramid_storage::wal::Wal;
use piramid_storage::SidecarManager;

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

pub fn save_index(storage: &Collection) -> Result<()> {
    SidecarManager::at(&storage.path).save_offsets(&storage.index)
}

pub fn save_vector_index(storage: &Collection) -> Result<()> {
    save_vec_idx(&storage.path, storage.vector_index.as_ref())
}

pub fn save_metadata(storage: &Collection) -> Result<()> {
    SidecarManager::at(&storage.path).save_manifest(&storage.metadata)
}

pub fn checkpoint(storage: &mut Collection) -> Result<()> {
    let timestamp = piramid_core::clock::unix_secs();

    // All three sidecars land before the WAL is cleared below, so a crash mid-checkpoint replays
    // rather than loses.
    save_index(storage)?;
    save_vector_index(storage)?;
    save_metadata(storage)?;

    if storage.config.wal.enabled {
        storage.checkpoint.wal.checkpoint(timestamp)?;
        storage.checkpoint.record_checkpoint(timestamp);
        let last_seq = storage.checkpoint.wal.next_seq.saturating_sub(1);
        SidecarManager::at(&storage.path).save_wal_meta(last_seq)?;
        storage.checkpoint.wal.rotate()?;
    }

    Ok(())
}

pub fn flush(storage: &mut Collection) -> Result<()> {
    storage.checkpoint.wal.flush()?;
    Ok(())
}
