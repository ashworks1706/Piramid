//! Vendor backend adapters.
//!
//! This module is the *only* place a vendor SDK type may appear. Everything above it — buffers,
//! streams, modules, kernels, and both the retrieval and inference layers — is written against
//! [`crate::device::DeviceRuntime`] and the free functions here.
//!
//! # Adding a backend
//!
//! Add a feature-gated submodule implementing [`DeviceRuntime`](crate::device::DeviceRuntime), then add one arm to each function
//! below. The dispatch is centralized here so no other file needs `#[cfg]` for hardware.

#[cfg(feature = "gpu-cuda")]
pub mod cudarc;

use crate::buffer::DeviceAllocation;
use crate::device::Device;
use crate::error::{GpuError, GpuResult};
use crate::stream::Stream;

/// Error returned by every entry point when no GPU backend is compiled in.
fn no_backend() -> GpuError {
    GpuError::Unavailable(
        "no GPU backend compiled in; rebuild with the `gpu-cuda` feature".to_string(),
    )
}

/// Open the default device for whichever backend this build enables.
pub fn open_default() -> GpuResult<Device> {
    #[cfg(feature = "gpu-cuda")]
    {
        cudarc::open_default()
    }
    #[cfg(not(feature = "gpu-cuda"))]
    {
        Err(no_backend())
    }
}

/// Allocate `size_bytes` of device memory.
pub fn allocate(_device: &Device, _size_bytes: usize) -> GpuResult<DeviceAllocation> {
    Err(no_backend())
}

/// Release a device allocation.
pub fn free(_device: &Device, _allocation: &mut DeviceAllocation) -> GpuResult<()> {
    Err(no_backend())
}

/// Copy host bytes into a device allocation.
pub fn copy_to_device(
    _device: &Device,
    _dst: &mut DeviceAllocation,
    _src: &[u8],
    _stream: &Stream,
) -> GpuResult<()> {
    Err(no_backend())
}

/// Copy device bytes back to the host.
pub fn copy_to_host(
    _device: &Device,
    _src: &DeviceAllocation,
    _dst: &mut [u8],
    _stream: &Stream,
) -> GpuResult<()> {
    Err(no_backend())
}

/// Create an independent execution stream, returning its backend identifier.
pub fn create_stream(_device: &Device) -> GpuResult<u64> {
    Err(no_backend())
}

/// Block until all work queued on `stream_id` completes.
pub fn synchronize_stream(_device: &Device, _stream_id: u64) -> GpuResult<()> {
    Err(no_backend())
}

/// Load a PTX image, returning its backend module identifier.
pub fn load_ptx(_device: &Device, _name: &'static str, _ptx: &str) -> GpuResult<u64> {
    Err(no_backend())
}
