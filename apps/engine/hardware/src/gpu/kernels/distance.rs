//! Batched distance kernels: one query against a contiguous slab of candidates. Not implemented;
//! `distance.cu` beside this file is where the device code goes.

use crate::gpu::buffer::DeviceBuffer;
use crate::gpu::error::{GpuError, GpuResult};
use crate::gpu::module::LaunchConfig;
use crate::gpu::stream::Stream;

/// Threads per block for the distance kernels; a starting point to tune once the kernel exists.
pub const BLOCK_SIZE: u32 = 256;

/// Arguments for one batched distance launch; buffers are borrowed so slabs can stay resident.
pub struct DistanceLaunch<'a> {
    /// Query vector, `dim` elements.
    pub query: &'a DeviceBuffer<f32>,
    /// Candidate slab, row-major, `rows * dim` elements.
    pub candidates: &'a DeviceBuffer<f32>,
    /// Output scores, one per row.
    pub out: &'a mut DeviceBuffer<f32>,
    /// Vector dimensionality.
    pub dim: usize,
    /// Number of candidate rows.
    pub rows: usize,
}

impl DistanceLaunch<'_> {
    /// Launch geometry for this batch: one thread per candidate row.
    pub fn launch_config(&self) -> LaunchConfig {
        LaunchConfig::for_elements(self.rows, BLOCK_SIZE)
    }
}

/// Launch the batched cosine-similarity kernel.
pub fn cosine_batch(_launch: DistanceLaunch<'_>, _stream: &Stream) -> GpuResult<()> {
    Err(GpuError::Launch(
        "cosine_batch kernel is not implemented".to_string(),
    ))
}

/// Launch the batched inner-product kernel.
pub fn dot_batch(_launch: DistanceLaunch<'_>, _stream: &Stream) -> GpuResult<()> {
    Err(GpuError::Launch(
        "dot_batch kernel is not implemented".to_string(),
    ))
}

/// Launch the batched squared-L2 kernel.
pub fn euclidean_batch(_launch: DistanceLaunch<'_>, _stream: &Stream) -> GpuResult<()> {
    Err(GpuError::Launch(
        "euclidean_batch kernel is not implemented".to_string(),
    ))
}
