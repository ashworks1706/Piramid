//! Candle-backed model execution.
//!
//! Compiled only under the `inference-candle` feature, so default builds pull in no model runtime.
//!
//! # Filling this in
//!
//! 1. Add `candle-core` and `candle-nn` to `Cargo.toml` under the `inference-candle` feature.
//! 2. Construct candle's device from the same [`piramid_gpu::Device`] the retrieval path holds — do
//!    not open a second context, or vectors and weights end up in separate allocations.
//! 3. Keep `candle` types inside this file.

use piramid_gpu::error::{GpuError, GpuResult};

/// Candle model-execution backend.
#[derive(Debug, Default)]
pub struct CandleBackend;

impl CandleBackend {
    /// Create the backend.
    pub fn new() -> Self {
        Self
    }

    /// Backend name, for logs and configuration.
    pub fn name(&self) -> &'static str {
        "candle"
    }

    /// Whether a model runtime is ready to serve.
    pub fn is_available(&self) -> bool {
        false
    }

    /// Initialize the backend against a device.
    pub fn initialize(&self) -> GpuResult<()> {
        Err(GpuError::Unavailable(
            "candle backend is scaffolded but no model runtime is wired yet".to_string(),
        ))
    }
}
