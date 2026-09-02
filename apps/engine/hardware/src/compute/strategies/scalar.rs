//! Portable scalar strategy: no intrinsics or threads, the correctness reference for the others.

use crate::compute::kernels::DistanceKernels;
use crate::compute::mode::ExecutionMode;

/// Scalar CPU kernels.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScalarStrategy;

impl DistanceKernels for ScalarStrategy {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Scalar
    }

    fn name(&self) -> &'static str {
        "scalar"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn cosine(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for (&x, &y) in a.iter().zip(b) {
            dot += x * y;
            norm_a += x * x;
            norm_b += y * y;
        }

        let denominator = norm_a.sqrt() * norm_b.sqrt();
        if denominator == 0.0 {
            0.0
        } else {
            dot / denominator
        }
    }

    fn dot(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut result = 0.0;
        for (&x, &y) in a.iter().zip(b) {
            result += x * y;
        }
        result
    }

    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32 {
        self.euclidean_squared(a, b).sqrt()
    }

    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut sum_sq = 0.0;
        for (&x, &y) in a.iter().zip(b) {
            let diff = x - y;
            sum_sq += diff * diff;
        }
        sum_sq
    }
}
