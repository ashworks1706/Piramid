use piramid_compute::Metric;
use piramid_core::config::ExecutionMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvfConfig {
    pub num_clusters: usize, // sqrt(N) is a good default
    pub num_probes: usize,   // clusters searched per query, 1-10, higher = better recall
    pub max_iterations: usize,
    pub metric: Metric,
    #[serde(default)]
    pub mode: ExecutionMode,
}

impl Default for IvfConfig {
    fn default() -> Self {
        IvfConfig {
            num_clusters: 100,
            num_probes: 5,
            max_iterations: 10,
            metric: Metric::Cosine,
            mode: ExecutionMode::default(),
        }
    }
}

impl IvfConfig {
    pub fn auto(num_vectors: usize) -> Self {
        let num_clusters = (num_vectors as f32).sqrt().max(10.0) as usize;
        let num_probes = (num_clusters as f32 * 0.1).clamp(1.0, 10.0) as usize;

        IvfConfig {
            num_clusters,
            num_probes,
            max_iterations: 10,
            metric: Metric::Cosine,
            mode: ExecutionMode::default(),
        }
    }
}
