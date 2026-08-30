//! Per-query recall and speed knobs.

use serde::{Deserialize, Serialize};

/// HNSW reads `ef`, IVF reads `nprobe`; flat search is always exhaustive and ignores both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SearchConfig {
    /// HNSW candidate list width. Falls back to the index's `ef_search`.
    pub ef: Option<usize>,

    /// IVF partitions to scan. Falls back to the index's `num_probes`.
    pub nprobe: Option<usize>,

    /// Multiplier on `k` when a filter is present, since some candidates get filtered out.
    #[serde(default = "default_filter_overfetch")]
    pub filter_overfetch: usize,

    /// Fan a batch of queries across the worker threads.
    #[serde(default = "crate::config::default_true")]
    pub parallel: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            ef: None,
            nprobe: None,
            filter_overfetch: default_filter_overfetch(),
            parallel: true,
        }
    }
}

fn default_filter_overfetch() -> usize {
    10
}
