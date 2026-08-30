use piramid_compute::Metric;
use piramid_core::config::ExecutionMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatConfig {
    pub metric: Metric,
    #[serde(default)]
    pub mode: ExecutionMode,
}
impl Default for FlatConfig {
    fn default() -> Self {
        FlatConfig {
            metric: Metric::Cosine,
            mode: ExecutionMode::default(),
        }
    }
}
