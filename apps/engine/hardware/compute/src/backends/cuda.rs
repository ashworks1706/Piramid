//! CUDA-backed distance kernels.
//!
//! The compute-layer adapter; it owns no CUDA types. Devices, buffers, streams and modules live
//! in `piramid-gpu` so `piramid-inference` can share the runtime without depending on `compute/`.
//!
//! Built only under `gpu-cuda`. Until kernels land, [`CudaBackend::is_available`] is `false` and
//! [`super::resolve_available`] falls back to CPU.
//!
//! To fill in: give [`CudaBackend`] a `OnceLock<Device>` probed in `is_available`, override the
//! `*_batch` methods with real launches, and leave the pairwise ones on CPU — a single-pair
//! distance will never pay for a launch.

use crate::backends::scalar::ScalarBackend;
use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;

/// GPU kernels dispatched through `piramid-gpu`.
#[derive(Debug, Default, Clone, Copy)]
pub struct CudaBackend;

impl CudaBackend {
    /// CPU backend used for single-pair work, which never justifies a launch.
    const PAIRWISE_FALLBACK: ScalarBackend = ScalarBackend;
}

impl DistanceKernels for CudaBackend {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Gpu
    }

    fn name(&self) -> &'static str {
        "cuda"
    }

    fn is_available(&self) -> bool {
        // No kernels are wired yet. Flipping this to a real device probe is step 1 above.
        false
    }

    fn cosine(&self, a: &[f32], b: &[f32]) -> f32 {
        Self::PAIRWISE_FALLBACK.cosine(a, b)
    }

    fn dot(&self, a: &[f32], b: &[f32]) -> f32 {
        Self::PAIRWISE_FALLBACK.dot(a, b)
    }

    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32 {
        Self::PAIRWISE_FALLBACK.euclidean(a, b)
    }

    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        Self::PAIRWISE_FALLBACK.euclidean_squared(a, b)
    }
}
