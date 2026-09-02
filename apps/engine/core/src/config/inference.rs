//! Inference settings.
//!
//! Nothing here is implemented yet. The shape is fixed now so the forward-pass work lands into a
//! settled surface, and [`InferenceConfig::validate`] refuses to start rather than accepting a
//! value it would ignore.

use serde::{Deserialize, Serialize};

/// Model execution: the forward pass, its cache, and how retrieval enters it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct InferenceConfig {
    /// Load a model at startup and serve `/api/infer`.
    pub enabled: bool,

    /// Directory or file holding the weights.
    pub model_path: Option<String>,

    /// Sequences batched into one forward pass.
    pub max_batch_size: usize,

    /// Longest prompt plus completion, in tokens.
    pub max_sequence_length: usize,

    pub kv_cache: KvCacheConfig,
    pub sampling: SamplingConfig,
    pub augment: AugmentConfig,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        InferenceConfig {
            enabled: false,
            model_path: None,
            max_batch_size: 8,
            max_sequence_length: 4096,
            kv_cache: KvCacheConfig::default(),
            sampling: SamplingConfig::default(),
            augment: AugmentConfig::default(),
        }
    }
}

/// Paged key/value cache for attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct KvCacheConfig {
    /// Total budget across every live sequence. `None` is unbounded.
    pub max_bytes: Option<u64>,

    /// Tokens per page. Pages are the unit of allocation and eviction.
    pub page_size: usize,
}

impl Default for KvCacheConfig {
    fn default() -> Self {
        KvCacheConfig {
            max_bytes: None,
            page_size: 16,
        }
    }
}

/// How the next token is drawn from the logits.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SamplingConfig {
    /// `0.0` is greedy.
    pub temperature: f32,

    /// Nucleus cutoff. `None` disables it.
    pub top_p: Option<f32>,

    /// Keep only this many highest-probability tokens. `None` disables it.
    pub top_k: Option<usize>,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        SamplingConfig {
            temperature: 0.0,
            top_p: None,
            top_k: None,
        }
    }
}

/// Retrieval fused into the forward pass, through `piramid_model::fusion::RetrievalHook`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AugmentConfig {
    /// Call the retrieval hook during generation.
    pub enabled: bool,

    /// Neighbours fetched per hook call.
    pub top_k: usize,
}

impl Default for AugmentConfig {
    fn default() -> Self {
        AugmentConfig {
            enabled: false,
            top_k: 8,
        }
    }
}

impl InferenceConfig {
    /// Reject anything the build cannot honour, rather than ignoring it.
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled {
            return Err("inference.enabled: not implemented yet (roadmap v0.4.0)".into());
        }
        if self.augment.enabled {
            return Err("inference.augment.enabled: not implemented yet (roadmap v0.4.0)".into());
        }
        Ok(())
    }
}
