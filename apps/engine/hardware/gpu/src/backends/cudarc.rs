//! NVIDIA CUDA backend, built on `cudarc`. Compiled only under `gpu-cuda`.
//!
//! To fill in: add `cudarc` under that feature, have [`CudaRuntime`] own the `CudaContext` and
//! populate [`DeviceCapabilities`] from a probe in [`open_default`], then implement the
//! allocation, transfer, stream and module functions in [`super`] by delegating here.
//!
//! `cudarc` types must not escape this file. That containment is what lets a second backend
//! (ROCm, Metal) land without touching anything above `gpu/`.

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
