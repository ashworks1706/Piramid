//! Build an index from its configuration.
//!
//! The configuration types live in `config/`; this is the only place that knows how to turn them
//! into concrete index instances.

use crate::traits::VectorIndex;
use crate::{FlatConfig, FlatIndex, HnswConfig, HnswIndex, IvfConfig, IvfIndex};
use piramid_core::config::{IndexConfig, IndexKind};

/// Construct the index `config` describes, sized for `num_vectors`.
pub fn create_index(config: &IndexConfig, num_vectors: usize) -> Box<dyn VectorIndex> {
    let index_type = config.select_type(num_vectors);

    match index_type {
        IndexKind::Flat => {
            // we use the metric and mode from the config, but the rest of the parameters are not needed for a flat index.
            let (metric, mode) = config.get_metric_and_mode();
            Box::new(FlatIndex::new(FlatConfig { metric, mode }))
        }
        IndexKind::Hnsw => {
            let config = match config {
                IndexConfig::Hnsw {
                    m,
                    m_max,
                    ef_construction,
                    ef_search,
                    ml,
                    metric,
                    mode,
                    ..
                } => HnswConfig {
                    m: *m,
                    m_max: *m_max,
                    ef_construction: *ef_construction,
                    ef_search: if *ef_search == 0 {
                        *ef_construction
                    } else {
                        *ef_search
                    },
                    ml: *ml,
                    metric: *metric,
                    mode: *mode,
                },
                _ => {
                    //  we use default HNSW parameters but apply the metric and mode from the config. The ef_search parameter defaults to the same value as ef_construction if not explicitly set
                    let (metric, mode) = config.get_metric_and_mode();
                    let auto = config.auto_config();
                    let m = auto.hnsw_m;
                    HnswConfig {
                        m,
                        m_max: m * 2,
                        ef_construction: auto.hnsw_ef_construction,
                        ef_search: auto.hnsw_ef_search,
                        ml: 1.0 / (m as f32).ln(),
                        metric,
                        mode,
                    }
                }
            };
            Box::new(HnswIndex::new(config))
        }
        IndexKind::Ivf => {
            let config = match config {
                IndexConfig::Ivf {
                    num_clusters,
                    num_probes,
                    max_iterations,
                    metric,
                    mode,
                    ..
                } => IvfConfig {
                    num_clusters: *num_clusters,
                    num_probes: *num_probes,
                    max_iterations: *max_iterations,
                    metric: *metric,
                    mode: *mode,
                },
                _ => {
                    // we use the auto-configure method to determine the number of clusters and probes based on the number of vectors, while applying the metric and mode from the config. configured dynamically based on the dataset size while still respecting user preferences for the distance metric and execution mode.
                    let (metric, mode) = config.get_metric_and_mode();
                    let auto = config.auto_config();
                    let mut config = IvfConfig::auto(num_vectors);
                    if let Some(num_clusters) = auto.ivf_num_clusters {
                        config.num_clusters = num_clusters;
                    }
                    if let Some(num_probes) = auto.ivf_num_probes {
                        config.num_probes = num_probes;
                    }
                    config.max_iterations = auto.ivf_max_iterations;
                    config.metric = metric;
                    config.mode = mode;
                    config
                }
            };
            Box::new(IvfIndex::new(config))
        }
    }
}
