//! NVIDIA CUDA backend, built on the `cudarc` crate.
//!
//! Compiled only under the `gpu-cuda` feature, so default builds need no CUDA toolkit.
//!
//! # Filling this in
//!
//! 1. Add `cudarc` to `Cargo.toml` under the `gpu-cuda` feature.
//! 2. Have [`CudaRuntime`] own the `CudaContext`, and populate [`DeviceCapabilities`] from a
//!    device probe in [`open_default`].
//! 3. Implement the allocation, transfer, stream, and module functions in
//!    [`super`] by delegating here.
//!
//! The `cudarc` types must not escape this file — that containment is what lets a second backend
//! (ROCm, Metal) be added later without touching anything above `gpu/`.

use std::sync::Arc;

use crate::device::{Device, DeviceCapabilities, DeviceRuntime};
use crate::error::{GpuError, GpuResult};

/// CUDA device runtime.
#[derive(Debug)]
pub struct CudaRuntime {
    capabilities: DeviceCapabilities,
}

impl CudaRuntime {
    /// Wrap probed device capabilities.
    pub fn new(capabilities: DeviceCapabilities) -> Self {
        Self { capabilities }
    }
}

impl DeviceRuntime for CudaRuntime {
    fn name(&self) -> &'static str {
        "cudarc"
    }

    fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }

    fn available_memory_bytes(&self) -> GpuResult<u64> {
        Err(GpuError::Unavailable(
            "cudarc backend is scaffolded but not wired to a device".to_string(),
        ))
    }

    fn synchronize(&self) -> GpuResult<()> {
        Err(GpuError::Unavailable(
            "cudarc backend is scaffolded but not wired to a device".to_string(),
        ))
    }
}

/// Probe and open the default CUDA device.
pub fn open_default() -> GpuResult<Device> {
    let _ = Arc::new(()); // placeholder: real impl constructs CudaContext here
    Err(GpuError::Unavailable(
        "cudarc backend is scaffolded but not wired to a device".to_string(),
    ))
}
