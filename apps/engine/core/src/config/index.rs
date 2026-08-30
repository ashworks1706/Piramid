//! Index selection configuration.

use serde::{Deserialize, Serialize};

use crate::config::SearchConfig;
use piramid_compute::{ExecutionMode, Metric};

/// Which index family a configuration resolves to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexKind {
    /// Brute-force scan.
    Flat,
    /// Graph index.
    Hnsw,
    /// Inverted-file index.
    Ivf,
}

/// Thresholds used by [`IndexConfig::Auto`] to pick a family by collection size.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AutoIndexConfig {
    #[serde(default = "default_flat_max_vectors")]
    pub flat_max_vectors: usize,
    #[serde(default = "default_ivf_max_vectors")]
    pub ivf_max_vectors: usize,
    #[serde(default)]
    pub ivf_num_clusters: Option<usize>,
    #[serde(default)]
    pub ivf_num_probes: Option<usize>,
    #[serde(default = "default_ivf_max_iterations")]
    pub ivf_max_iterations: usize,
    #[serde(default = "default_hnsw_m")]
    pub hnsw_m: usize,
    #[serde(default = "default_hnsw_ef_construction")]
    pub hnsw_ef_construction: usize,
    #[serde(default = "default_hnsw_ef_search")]
    pub hnsw_ef_search: usize,
}

impl Default for AutoIndexConfig {
    fn default() -> Self {
        Self {
            flat_max_vectors: default_flat_max_vectors(),
            ivf_max_vectors: default_ivf_max_vectors(),
            ivf_num_clusters: None,
            ivf_num_probes: None,
            ivf_max_iterations: default_ivf_max_iterations(),
            hnsw_m: default_hnsw_m(),
            hnsw_ef_construction: default_hnsw_ef_construction(),
            hnsw_ef_search: default_hnsw_ef_search(),
        }
    }
}

/// Index configuration for a collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IndexConfig {
    /// Pick a family from the collection's size.
    Auto {
        metric: Metric,
        #[serde(default)]
        mode: ExecutionMode,
        #[serde(default)]
        search: SearchConfig,
        #[serde(default)]
        auto: AutoIndexConfig,
    },
    Flat {
        metric: Metric,
        #[serde(default)]
        mode: ExecutionMode,
        #[serde(default)]
        search: SearchConfig,
    },
    Hnsw {
        m: usize,
        m_max: usize,
        ef_construction: usize,
        ef_search: usize,
        ml: f32,
        metric: Metric,
        #[serde(default)]
        mode: ExecutionMode,
        #[serde(default)]
        search: SearchConfig,
    },
    Ivf {
        num_clusters: usize,
        num_probes: usize,
        max_iterations: usize,
        metric: Metric,
        #[serde(default)]
        mode: ExecutionMode,
        #[serde(default)]
        search: SearchConfig,
    },
}

impl Default for IndexConfig {
    fn default() -> Self {
        IndexConfig::Auto {
            metric: Metric::Cosine,
            mode: ExecutionMode::default(),
            search: SearchConfig::default(),
            auto: AutoIndexConfig::default(),
        }
    }
}

impl IndexConfig {
    /// Pick a family for a collection of `num_vectors`.
    pub fn select_type(&self, num_vectors: usize) -> IndexKind {
        match self {
            IndexConfig::Auto { auto, .. } => {
                if num_vectors < auto.flat_max_vectors {
                    IndexKind::Flat
                } else if num_vectors < auto.ivf_max_vectors {
                    IndexKind::Ivf
                } else {
                    IndexKind::Hnsw
                }
            }
            IndexConfig::Flat { .. } => IndexKind::Flat,
            IndexConfig::Hnsw { .. } => IndexKind::Hnsw,
            IndexConfig::Ivf { .. } => IndexKind::Ivf,
        }
    }

    /// Metric and execution mode shared by every variant.
    pub fn get_metric_and_mode(&self) -> (Metric, ExecutionMode) {
        match self {
            IndexConfig::Auto { metric, mode, .. }
            | IndexConfig::Flat { metric, mode, .. }
            | IndexConfig::Hnsw { metric, mode, .. }
            | IndexConfig::Ivf { metric, mode, .. } => (*metric, *mode),
        }
    }

    /// Per-query recall/speed knobs.
    pub fn search_config(&self) -> SearchConfig {
        match self {
            IndexConfig::Auto { search, .. }
            | IndexConfig::Flat { search, .. }
            | IndexConfig::Hnsw { search, .. }
            | IndexConfig::Ivf { search, .. } => *search,
        }
    }

    /// Auto-selection thresholds, defaulted for explicit variants.
    pub fn auto_config(&self) -> AutoIndexConfig {
        match self {
            IndexConfig::Auto { auto, .. } => *auto,
            _ => AutoIndexConfig::default(),
        }
    }
}

fn default_flat_max_vectors() -> usize {
    10_000
}

fn default_ivf_max_vectors() -> usize {
    100_000
}

fn default_ivf_max_iterations() -> usize {
    10
}

fn default_hnsw_m() -> usize {
    16
}

fn default_hnsw_ef_construction() -> usize {
    200
}

fn default_hnsw_ef_search() -> usize {
    200
}
