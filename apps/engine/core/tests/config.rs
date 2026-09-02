#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "assertions in tests"
)]

use piramid_core::config::{
    AutoIndexConfig, Config, HardwareProfile, IndexConfig, IndexKind, LogLevel, QuantizationLevel,
    QuantizationStage,
};
use piramid_hardware::compute::Metric;

#[test]
fn the_default_config_round_trips_through_yaml() {
    let cfg = Config::default();
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    let parsed: Config = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(cfg, parsed);
    assert!(yaml.contains("startup:"));
    assert!(yaml.contains("runtime:"));
}

#[test]
fn an_empty_file_is_all_defaults() {
    let cfg: Config = serde_yaml::from_str("{}").unwrap();

    assert_eq!(cfg, Config::default());
    assert_eq!(cfg.startup.bind, "0.0.0.0:6333");
    assert_eq!(cfg.startup.hardware.profile, HardwareProfile::Auto);
    assert_eq!(cfg.startup.logging.level, LogLevel::Info);
    assert_eq!(cfg.runtime.quantization.stage, QuantizationStage::Disabled);
    assert_eq!(cfg.runtime.search.filter_overfetch, 10);
    cfg.validate().unwrap();
}

#[test]
fn a_partial_file_defaults_the_rest() {
    let yaml = r"
startup:
  bind: 127.0.0.1:7000
runtime:
  search:
    filter_overfetch: 3
";
    let cfg: Config = serde_yaml::from_str(yaml).unwrap();

    assert_eq!(cfg.startup.bind, "127.0.0.1:7000");
    assert_eq!(cfg.runtime.search.filter_overfetch, 3);
    assert!(cfg.runtime.search.parallel);
    assert_eq!(cfg.startup.logging.level, LogLevel::Info);
    cfg.validate().unwrap();
}

#[test]
fn a_misspelled_key_is_an_error_rather_than_a_silent_default() {
    let yaml = r"
runtime:
  search:
    filter_overfech: 3
";
    let err = serde_yaml::from_str::<Config>(yaml)
        .unwrap_err()
        .to_string();
    assert!(err.contains("filter_overfech"), "{err}");
}

#[test]
fn a_setting_in_the_wrong_block_is_an_error() {
    let yaml = r"
runtime:
  bind: 127.0.0.1:7000
";
    assert!(serde_yaml::from_str::<Config>(yaml).is_err());
}

#[test]
fn quantization_can_express_pre_and_post_search_experiments() {
    let mut cfg = Config::default();
    cfg.runtime.quantization.level = QuantizationLevel::Int8;
    cfg.runtime.quantization.stage = QuantizationStage::QueryPreSearch;
    cfg.validate().unwrap();

    cfg.runtime.quantization = cfg.runtime.quantization.post_search();
    assert_eq!(
        cfg.runtime.quantization.stage,
        QuantizationStage::ResultPostSearch
    );
    cfg.validate().unwrap();
}

#[test]
fn auto_index_thresholds_are_configurable() {
    let cfg = IndexConfig::Auto {
        metric: Metric::Cosine,
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
fn unimplemented_settings_are_rejected_rather_than_ignored() {
    let mut cfg = Config::default();

    cfg.runtime.quantization.level = QuantizationLevel::Int4;
    assert!(cfg.validate().is_err());
    cfg.runtime.quantization.level = QuantizationLevel::Float16;
    assert!(cfg.validate().is_err());

    let mut cfg = Config::default();
    cfg.runtime.inference.enabled = true;
    let err = cfg.validate().unwrap_err();
    assert!(err.contains("not implemented"), "{err}");

    let mut cfg = Config::default();
    cfg.runtime.inference.augment.enabled = true;
    assert!(cfg.validate().unwrap_err().contains("not implemented"));
}

#[test]
fn a_bad_bind_address_is_rejected() {
    let mut cfg = Config::default();
    cfg.startup.bind = "6333".to_string();
    assert!(cfg.validate().unwrap_err().contains("address:port"));
}
