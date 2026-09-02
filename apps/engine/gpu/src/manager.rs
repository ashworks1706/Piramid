//! The GPU domain entry: opens devices and hands out the handles everything else borrows.

use crate::device::Device;
use crate::error::GpuResult;
use crate::stream::Stream;

/// Owns device acquisition for the process.
///
/// Callers go through this to get a [`Device`]; the resources it hands out — [`Device`],
/// [`crate::DeviceBuffer`], [`Stream`] — keep their own names. Multi-device policy (which
/// ordinal, memory admission) lands here when a real backend does.
#[derive(Debug)]
pub struct GpuManager {
    device: Device,
}

impl GpuManager {
    /// Open the default device; errors when no GPU backend is compiled in or none is present.
    pub fn open() -> GpuResult<Self> {
        Ok(Self {
            device: Device::open_default()?,
        })
    }

    /// The device this manager opened.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Create a stream on the managed device.
    pub fn stream(&self) -> GpuResult<Stream> {
        Stream::new(&self.device)
    }
}
