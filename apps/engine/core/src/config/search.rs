//! Per-query recall and speed knobs.

use serde::{Deserialize, Serialize};

/// HNSW reads `ef`, IVF reads `nprobe`; flat search is always exhaustive and ignores both.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SearchConfig {
    /// HNSW candidate list width. Falls back to the index's `ef_search`.
    pub ef: Option<usize>,

    /// IVF partitions to scan. Falls back to the index's `num_probes`.
    pub nprobe: Option<usize>,

    /// Multiplier on `k` when a filter is present, since some candidates get filtered out.
    #[serde(default = "default_filter_overfetch")]
    pub filter_overfetch: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            ef: None,
            nprobe: None,
            filter_overfetch: default_filter_overfetch(),
        }
    }
}

fn default_filter_overfetch() -> usize {
    10
}
