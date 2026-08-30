//! Vector compression settings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QuantizationLevel {
    /// Full-precision f32.
    #[default]
    None,
    /// 8-bit integer, scaled per vector.
    Int8,
    /// Product quantization with `subquantizers` blocks.
    Pq { subquantizers: usize },
    /// 4-bit integer. Not implemented; rejected by `AppConfig::validate`.
    Int4,
    /// Half precision. Not implemented; rejected by `AppConfig::validate`.
    Float16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuantizationStage {
    #[default]
    Disabled,
    Storage,
    Index,
    QueryPreSearch,
    ResultPostSearch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuantizationConfig {
    pub level: QuantizationLevel,

    /// Compress on disk only. `false` also quantizes the in-memory copy.
    pub disk_only: bool,

    #[serde(default)]
    pub stage: QuantizationStage,

    #[serde(default = "default_preserve_raw_vectors")]
    pub preserve_raw_vectors: bool,

    #[serde(default)]
    pub storage_enabled: bool,

    #[serde(default)]
    pub index_enabled: bool,

    #[serde(default)]
    pub query_enabled: bool,

    #[serde(default)]
    pub result_enabled: bool,
}

impl Default for QuantizationConfig {
    fn default() -> Self {
        QuantizationConfig {
            level: QuantizationLevel::None,
            disk_only: false,
            stage: QuantizationStage::Disabled,
            preserve_raw_vectors: true,
            storage_enabled: false,
            index_enabled: false,
            query_enabled: false,
            result_enabled: false,
        }
    }
}

impl QuantizationConfig {
    /// Quantize for index and search while keeping raw vectors on disk.
    pub fn int8() -> Self {
        QuantizationConfig {
            level: QuantizationLevel::Int8,
            disk_only: false,
            stage: QuantizationStage::Index,
            preserve_raw_vectors: true,
            storage_enabled: false,
            index_enabled: true,
            query_enabled: false,
            result_enabled: false,
        }
    }

    pub fn int8_disk_only() -> Self {
        QuantizationConfig {
            level: QuantizationLevel::Int8,
            disk_only: true,
            stage: QuantizationStage::Storage,
            preserve_raw_vectors: false,
            storage_enabled: true,
            index_enabled: false,
            query_enabled: false,
            result_enabled: false,
        }
    }

    pub fn pq(subquantizers: usize) -> Self {
        QuantizationConfig {
            level: QuantizationLevel::Pq { subquantizers },
            disk_only: false,
            stage: QuantizationStage::Index,
            preserve_raw_vectors: true,
            storage_enabled: false,
            index_enabled: true,
            query_enabled: false,
            result_enabled: false,
        }
    }

    pub fn pre_search(mut self) -> Self {
        self.stage = QuantizationStage::QueryPreSearch;
        self.query_enabled = true;
        self
    }

    pub fn post_search(mut self) -> Self {
        self.stage = QuantizationStage::ResultPostSearch;
        self.result_enabled = true;
        self
    }
}

fn default_preserve_raw_vectors() -> bool {
    true
}
