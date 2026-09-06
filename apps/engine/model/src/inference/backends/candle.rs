//! Candle-backed model execution. Compiled only under inference-candle.

use piramid_hardware::gpu::error::{GpuError, GpuResult};

/// Candle model-execution runtime, shaped like gpu::DeviceRuntime for models.
#[derive(Debug, Default)]
pub struct CandleRuntime;

impl CandleRuntime {
    /// Create the runtime.
    pub fn new() -> Self {
        Self
    }

    /// Runtime name, for logs and configuration.
    pub fn name(&self) -> &'static str {
        "candle"
    }

    /// Whether a model runtime is ready to serve.
    pub fn is_available(&self) -> bool {
        false
    }

    /// Initialize the runtime against a device.
    pub fn initialize(&self) -> GpuResult<()> {
        Err(GpuError::Unavailable(
            "candle runtime is scaffolded but no model runtime is wired yet".to_string(),
        ))
    }
}
