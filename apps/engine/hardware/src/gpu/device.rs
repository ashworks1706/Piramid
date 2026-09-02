//! Device discovery, capabilities, and the runtime handle.

use std::sync::Arc;

use crate::gpu::error::GpuResult;

/// What a device can do, probed once at startup and consulted before selecting a kernel variant.
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

/// The contract a device runtime must satisfy, implemented per vendor backend under [`crate::gpu::backends`].
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

/// A cheap-to-clone handle to one compute device, shared by the retrieval and inference paths.
#[derive(Debug, Clone)]
pub struct Device {
    runtime: Arc<dyn DeviceRuntime>,
}

impl Device {
    /// Wrap a backend runtime in a shareable handle.
    pub fn new(runtime: Arc<dyn DeviceRuntime>) -> Self {
        Self { runtime }
    }

    /// Open the default device for this build; unavailable when no GPU backend is compiled in.
    pub fn open_default() -> GpuResult<Self> {
        crate::gpu::backends::open_default()
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
