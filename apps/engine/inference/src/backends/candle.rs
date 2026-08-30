//! Candle-backed model execution. Compiled only under `inference-candle`.

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
