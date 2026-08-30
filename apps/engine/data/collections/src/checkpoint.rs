//! Checkpoint bookkeeping: when to flush sidecars, and clearing the WAL once they are durable.

use super::collection::Collection;
use piramid_core::error::Result;
use piramid_index::save_vector_index as save_vec_idx;
use piramid_storage::persistence::{save_index as save_idx, save_metadata as save_meta};
use piramid_storage::wal::Wal;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

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
    save_idx(&storage.path, &storage.index)
}

pub fn save_vector_index(storage: &Collection) -> Result<()> {
    save_vec_idx(&storage.path, storage.vector_index.as_ref())
}

pub fn save_metadata(storage: &Collection) -> Result<()> {
    save_meta(&storage.path, &storage.metadata)
}

fn wal_meta_path(path: &str) -> PathBuf {
    PathBuf::from(format!("{}.wal.meta", path))
}

#[derive(Serialize, Deserialize, Default)]
struct WalMeta {
    last_checkpoint_seq: u64,
}

pub fn load_wal_meta(path: &str) -> Result<u64> {
    let meta_path = wal_meta_path(path);
    let data = match fs::read(&meta_path) {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    let meta: WalMeta = serde_json::from_slice(&data)?;
    Ok(meta.last_checkpoint_seq)
}

fn save_wal_meta(path: &str, last_checkpoint_seq: u64) -> Result<()> {
    let meta_path = wal_meta_path(path);
    let tmp_path = meta_path.with_extension("tmp");
    let meta = WalMeta {
        last_checkpoint_seq,
    };
    fs::write(&tmp_path, serde_json::to_vec(&meta)?)?;
    fs::rename(&tmp_path, &meta_path)?;
    let file = fs::File::open(&meta_path)?;
    file.sync_all()?;
    Ok(())
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
        save_wal_meta(&storage.path, last_seq)?;
        storage.checkpoint.wal.rotate()?;
    }

    Ok(())
}

pub fn flush(storage: &mut Collection) -> Result<()> {
    storage.checkpoint.wal.flush()?;
    Ok(())
}
