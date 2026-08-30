//! CUDA-backed distance kernels.
//!
//! The compute-layer adapter; it owns no CUDA types. Devices, buffers, streams and modules live
//! in `piramid-gpu` so `piramid-inference` can share the runtime without depending on `compute/`.
//!
//! Built only under `gpu-cuda`. Until kernels land, [`CudaBackend::is_available`] is `false`, so
//! [`super::for_mode`] refuses `ExecutionMode::Gpu` rather than quietly serving something else.
//!
//! To fill in: give [`CudaBackend`] a `OnceLock<Device>` probed in `is_available` and override the
//! `*_batch` methods with real launches. The pairwise methods stay on the CPU by design — a
//! single-pair distance will never pay for a kernel launch — which is a dispatch decision, not a
//! fallback: this backend is the GPU one for batches and the CPU one for pairs, always, on every
//! machine. There is no configuration under which it silently changes its mind.

use crate::backends::scalar::ScalarBackend;
use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;

/// GPU kernels dispatched through `piramid-gpu`.
#[derive(Debug, Default, Clone, Copy)]
pub struct CudaBackend;

impl CudaBackend {
    /// Single-pair work runs here unconditionally; see the module docs.
    const PAIRWISE: ScalarBackend = ScalarBackend;
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
        Self::PAIRWISE.cosine(a, b)
    }

    fn dot(&self, a: &[f32], b: &[f32]) -> f32 {
        Self::PAIRWISE.dot(a, b)
    }

    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32 {
        Self::PAIRWISE.euclidean(a, b)
    }

    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        Self::PAIRWISE.euclidean_squared(a, b)
    }
}
