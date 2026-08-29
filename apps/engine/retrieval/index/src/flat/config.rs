// Flat index configuration
use piramid_compute::Metric;
use piramid_core::config::ExecutionMode;
use serde::{Deserialize, Serialize};

// Flat index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatConfig {
    pub metric: Metric, // Distance metric to use for similarity calculations (e.g., cosine, euclidean)
    #[serde(default)]
    pub mode: ExecutionMode, // Execution mode for search operations (e.g., auto, single-threaded, multi-threaded)
}
impl Default for FlatConfig {
    fn default() -> Self {
        FlatConfig {
            metric: Metric::Cosine,
            mode: ExecutionMode::default(),
        }
    }
}
