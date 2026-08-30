#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "assertions in tests"
)]

use piramid_compute::Metric;
use piramid_core::config::{
    AppConfig, AutoIndexConfig, HardwareProfile, IndexConfig, IndexKind, LogLevel,
    QuantizationLevel, QuantizationStage,
};

#[test]
fn app_config_serializes_expanded_research_knobs() {
    let cfg = AppConfig::default();
    let yaml = serde_yaml::to_string(&cfg).unwrap();

    assert!(yaml.contains("hardware:"));
    assert!(yaml.contains("logging:"));
    assert!(yaml.contains("preserve_raw_vectors: true"));
}

#[test]
fn minimal_config_files_receive_defaults_for_new_knobs() {
    let yaml = r#"
index:
  type: Auto
  metric: Cosine
quantization:
  level: None
  disk_only: false
memory:
  initial_mmap_size: 1048576
  use_mmap: true
wal:
  enabled: true
  checkpoint_frequency: 1000
  max_log_size: 1048576
  sync_on_write: false
parallelism:
  mode: Auto
  parallel_search: true
execution: Auto
search:
  filter_overfetch: 10
limits: {}
"#;

    let cfg: AppConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.hardware.profile, HardwareProfile::Auto);
    assert_eq!(cfg.logging.level, LogLevel::Info);
    assert_eq!(cfg.quantization.stage, QuantizationStage::Disabled);
    assert!(cfg.quantization.preserve_raw_vectors);
    assert_eq!(cfg.search.filter_overfetch, 10);
    cfg.validate().unwrap();
}

#[test]
fn quantization_can_express_pre_and_post_search_experiments() {
    let mut cfg = AppConfig::default();
    cfg.quantization.level = QuantizationLevel::Int8;
    cfg.quantization.stage = QuantizationStage::QueryPreSearch;
    cfg.quantization.query_enabled = true;
    cfg.validate().unwrap();

    cfg.quantization = cfg.quantization.post_search();
    assert_eq!(cfg.quantization.stage, QuantizationStage::ResultPostSearch);
    assert!(cfg.quantization.result_enabled);
    cfg.validate().unwrap();
}

#[test]
fn auto_index_thresholds_are_configurable() {
    let cfg = IndexConfig::Auto {
        metric: Metric::Cosine,
        mode: piramid_core::config::ExecutionMode::Auto,
        search: piramid_core::config::SearchConfig::default(),
        auto: AutoIndexConfig {
            flat_max_vectors: 5,
            ivf_max_vectors: 10,
            ivf_num_clusters: Some(3),
            ivf_num_probes: Some(2),
            ivf_max_iterations: 4,
            hnsw_m: 8,
            hnsw_ef_construction: 64,
            hnsw_ef_search: 32,
        },
    };

    assert_eq!(cfg.select_type(4), IndexKind::Flat);
    assert_eq!(cfg.select_type(7), IndexKind::Ivf);
    assert_eq!(cfg.select_type(12), IndexKind::Hnsw);
}

#[test]
fn unimplemented_quantization_levels_fail_validation() {
    let mut cfg = AppConfig::default();
    cfg.quantization.level = QuantizationLevel::Int4;
    assert!(cfg.validate().is_err());

    cfg.quantization.level = QuantizationLevel::Float16;
    assert!(cfg.validate().is_err());
}
