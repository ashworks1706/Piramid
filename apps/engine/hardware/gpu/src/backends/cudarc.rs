//! NVIDIA CUDA backend, built on `cudarc`; compiled only under `gpu-cuda`. `cudarc` types must
//! never escape this file.

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
