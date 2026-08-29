//! CUDA-backed distance kernels.
//!
//! This is the compute-layer *adapter*: it owns no CUDA types itself. Device handles, buffers,
//! streams, and module loading all live in `piramid-gpu`, so that `piramid-inference` can share
//! the same device runtime without depending on `compute/`.
//!
//! Compiled only under the `gpu-cuda` feature. Until kernels land, [`CudaBackend::is_available`]
//! reports `false`, so [`super::resolve_available`] transparently falls back to a CPU backend and
//! nothing in the query path breaks.
//!
//! # Filling this in
//!
//! 1. Give [`CudaBackend`] a `OnceLock<Device>` and probe it in `is_available`.
//! 2. Override `cosine_batch` / `dot_batch` / `euclidean_batch` with real launches — those take a
//!    contiguous slab precisely so they can be uploaded in one copy.
//! 3. Leave the pairwise methods delegating to the CPU backend. A single-pair distance will never
//!    be worth a kernel launch; the batch path is the one that pays.

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
