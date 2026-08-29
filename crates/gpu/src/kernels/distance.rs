//! Batched distance kernels.
//!
//! Scores one query against a contiguous slab of candidate vectors. Mirrors the batch methods on
//! [`piramid_compute::DistanceKernels`], which is why that trait takes a slab: the slab uploads in
//! one transfer and indexes directly as a device-side 2-D array.
//!
//! Not yet implemented. `distance.cu` alongside this file is the intended home for the device
//! code; this wrapper builds the launch geometry and binds arguments.

use crate::buffer::DeviceBuffer;
use crate::error::{GpuError, GpuResult};
use crate::module::LaunchConfig;
use crate::stream::Stream;

/// Threads per block for the distance kernels.
///
/// 256 is a reasonable starting point on most NVIDIA parts; tune against `benches/` once the
/// kernel exists.
pub const BLOCK_SIZE: u32 = 256;

/// Arguments for one batched distance launch.
///
/// Borrowing the buffers rather than owning them is deliberate: the candidate slab is expected to
/// stay resident across many queries.
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
