//! CUDA-backed distance kernels; the compute-layer adapter, owns no CUDA types itself.

use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;
use crate::strategies::scalar::ScalarBackend;

/// GPU kernels dispatched through `piramid-gpu`.
#[derive(Debug, Default, Clone, Copy)]
pub struct CudaBackend;

impl CudaBackend {
    /// Single-pair work always runs on the CPU; a launch never pays off for one vector.
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
        // No kernels are wired yet; this stays false until a real device probe lands.
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
