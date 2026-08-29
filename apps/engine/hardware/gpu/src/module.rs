//! Compiled kernel modules.
//!
//! A [`KernelModule`] is a loaded PTX/cubin image; a [`LaunchConfig`] is the geometry one launch
//! runs with. Keeping both here means kernel *sources* under [`crate::kernels`] stay pure
//! device code with a thin typed wrapper, and nothing above has to know how a module was built.

use crate::device::Device;
use crate::error::GpuResult;

/// Grid and block geometry for a single kernel launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchConfig {
    /// Blocks per grid, in `(x, y, z)`.
    pub grid: (u32, u32, u32),
    /// Threads per block, in `(x, y, z)`.
    pub block: (u32, u32, u32),
    /// Dynamically allocated shared memory per block, in bytes.
    pub shared_memory_bytes: u32,
}

impl LaunchConfig {
    /// One-dimensional geometry covering `n` elements at `block_size` threads per block.
    pub fn for_elements(n: usize, block_size: u32) -> Self {
        let block_size = block_size.max(1);
        let blocks = n.div_ceil(block_size as usize).max(1) as u32;
        Self {
            grid: (blocks, 1, 1),
            block: (block_size, 1, 1),
            shared_memory_bytes: 0,
        }
    }
}

/// A compiled device module, loaded once and reused across launches.
#[derive(Debug)]
pub struct KernelModule {
    device: Device,
    name: &'static str,
    /// Backend module identifier.
    id: u64,
}

impl KernelModule {
    /// Load a module from a compiled PTX image.
    pub fn load_ptx(device: &Device, name: &'static str, ptx: &str) -> GpuResult<Self> {
        let id = crate::backends::load_ptx(device, name, ptx)?;
        Ok(Self {
            device: device.clone(),
            name,
            id,
        })
    }

    /// Module name, as used in logs and error messages.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Backend module identifier.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Device this module is loaded on.
    pub fn device(&self) -> &Device {
        &self.device
    }
}
