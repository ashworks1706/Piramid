//! Candle-backed model execution. Compiled only under `inference-candle`.
//!
//! To fill in: add `candle-core` and `candle-nn` under that feature, construct candle's device
//! from the same `piramid-gpu::Device` the retrieval path holds (a second context puts vectors
//! and weights in separate allocations), and keep `candle` types inside this file.

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
