//! Vector compression settings.

use serde::{Deserialize, Serialize};

/// Which compression a collection asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QuantizationLevel {
    /// Full-precision f32.
    #[default]
    None,
    /// 8-bit integer, scaled per vector.
    Int8,
    /// Product quantization with `subquantizers` blocks.
    Pq {
        /// Number of blocks the vector is split into.
        subquantizers: usize,
    },
    /// 4-bit integer. Not implemented; rejected by `AppConfig::validate`.
    Int4,
    /// Half precision. Not implemented; rejected by `AppConfig::validate`.
    Float16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
/// Which point in the pipeline quantization applies at.
pub enum QuantizationStage {
    /// No quantization anywhere.
    #[default]
    Disabled,
    /// Quantize what is written to disk.
    Storage,
    /// Quantize what the index scores against.
    Index,
    /// Quantize the query before searching.
    QueryPreSearch,
    /// Quantize results after searching.
    ResultPostSearch,
}

/// Where and how aggressively vectors are compressed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuantizationConfig {
    /// The encoding to use.
    pub level: QuantizationLevel,

    /// Compress on disk only. `false` also quantizes the in-memory copy.
    pub disk_only: bool,

    /// Where in the pipeline the encoding applies.
    #[serde(default)]
    pub stage: QuantizationStage,

    /// Keep full-precision vectors alongside the quantized copies.
    #[serde(default = "default_preserve_raw_vectors")]
    pub preserve_raw_vectors: bool,

    /// Quantize what is written to disk.
    #[serde(default)]
    pub storage_enabled: bool,

    /// Quantize what the index scores against.
    #[serde(default)]
    pub index_enabled: bool,

    /// Quantize the query before searching.
    #[serde(default)]
    pub query_enabled: bool,

    /// Quantize results after searching.
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

    /// Product quantization for the index, raw vectors kept on disk.
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

    /// Switch the stage to quantizing results after search.
    pub fn post_search(mut self) -> Self {
        self.stage = QuantizationStage::ResultPostSearch;
        self.result_enabled = true;
        self
    }
}

fn default_preserve_raw_vectors() -> bool {
    true
}
