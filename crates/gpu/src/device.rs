//! Device discovery, capabilities, and the runtime handle.

use std::sync::Arc;

use crate::error::GpuResult;

/// What a device can do, probed once at startup.
///
/// Consulted before selecting a kernel variant — e.g. tensor-core paths need a minimum compute
/// capability, and batch sizing depends on available memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCapabilities {
    /// Human-readable device name.
    pub name: String,
    /// Zero-based device ordinal.
    pub ordinal: usize,
    /// CUDA compute capability as `(major, minor)`.
    pub compute_capability: (u32, u32),
    /// Total device memory in bytes.
    pub total_memory_bytes: u64,
    /// Multiprocessor count, for occupancy and launch-geometry decisions.
    pub multiprocessor_count: u32,
}

/// The contract a device runtime must satisfy.
///
/// Implemented per vendor backend under [`crate::backends`]. Everything above this trait —
/// [`piramid_compute`] kernels and [`piramid_inference`] model execution alike — is written against
/// the trait, never against a vendor SDK.
pub trait DeviceRuntime: Send + Sync + std::fmt::Debug {
    /// Backend name, e.g. `"cudarc"`.
    fn name(&self) -> &'static str;

    /// Capabilities of the selected device.
    fn capabilities(&self) -> &DeviceCapabilities;

    /// Free device memory in bytes, for admission control and batch sizing.
    fn available_memory_bytes(&self) -> GpuResult<u64>;

    /// Block until all queued work on this device completes.
    fn synchronize(&self) -> GpuResult<()>;
}

/// A handle to one compute device.
///
/// Cheap to clone and shared across subsystems: the retrieval path and the inference path are
/// expected to hold the *same* `Device` so vectors and model weights land in one address space
/// and never round-trip through the host between them.
#[derive(Debug, Clone)]
pub struct Device {
    runtime: Arc<dyn DeviceRuntime>,
}

impl Device {
    /// Wrap a backend runtime in a shareable handle.
    pub fn new(runtime: Arc<dyn DeviceRuntime>) -> Self {
        Self { runtime }
    }

    /// Open the default device for this build.
    ///
    /// Returns [`GpuError::Unavailable`](crate::GpuError::Unavailable) when no GPU backend is
    /// compiled in, so callers can fall back to CPU without conditional compilation of their own.
    pub fn open_default() -> GpuResult<Self> {
        crate::backends::open_default()
    }

    /// Borrow the underlying runtime.
    pub fn runtime(&self) -> &Arc<dyn DeviceRuntime> {
        &self.runtime
    }

    /// Capabilities of this device.
    pub fn capabilities(&self) -> &DeviceCapabilities {
        self.runtime.capabilities()
    }

    /// Block until all queued work completes.
    pub fn synchronize(&self) -> GpuResult<()> {
        self.runtime.synchronize()
    }
}
