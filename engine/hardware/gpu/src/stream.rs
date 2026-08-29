//! Execution streams.
//!
//! A [`Stream`] is an ordered queue of device work. Separate streams overlap: uploading the next
//! batch of candidate vectors while the current kernel still runs is the difference between a GPU
//! path that beats CPU and one that does not.

use crate::device::Device;
use crate::error::GpuResult;

/// An ordered queue of device operations.
#[derive(Debug, Clone)]
pub struct Stream {
    device: Device,
    /// Backend stream identifier; `0` is the default stream.
    id: u64,
}

impl Stream {
    /// The device's default (synchronizing) stream.
    pub fn default_for(device: &Device) -> Self {
        Self {
            device: device.clone(),
            id: 0,
        }
    }

    /// Create an independent stream that can overlap with others.
    pub fn new(device: &Device) -> GpuResult<Self> {
        let id = crate::backends::create_stream(device)?;
        Ok(Self {
            device: device.clone(),
            id,
        })
    }

    /// Backend stream identifier.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Device this stream belongs to.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Block until every operation queued on this stream has completed.
    pub fn synchronize(&self) -> GpuResult<()> {
        crate::backends::synchronize_stream(&self.device, self.id)
    }
}
