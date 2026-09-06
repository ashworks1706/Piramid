//! CUDA-backed distance kernels; the compute-layer adapter, owns no CUDA types itself.

use crate::compute::kernels::DistanceKernels;
use crate::compute::mode::ExecutionMode;
use crate::compute::strategies::scalar::ScalarStrategy;

/// GPU kernels dispatched through the gpu module.
#[derive(Debug, Default, Clone, Copy)]
pub struct CudaStrategy;

impl CudaStrategy {
    /// Single-pair work runs on the CPU.
    const PAIRWISE: ScalarStrategy = ScalarStrategy;
}

impl DistanceKernels for CudaStrategy {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Gpu
    }

    fn name(&self) -> &'static str {
        "cuda"
    }

    fn is_available(&self) -> bool {
        // No kernels are wired yet.
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
