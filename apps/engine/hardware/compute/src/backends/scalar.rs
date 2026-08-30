//! Portable scalar backend.
//!
//! No intrinsics, no threads. The correctness reference the other backends are checked against,
//! and the fallback where there is no vector unit.

use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;

/// Scalar CPU kernels.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScalarBackend;

impl DistanceKernels for ScalarBackend {
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

        for i in 0..a.len() {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
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
        for i in 0..a.len() {
            result += a[i] * b[i];
        }
        result
    }

    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32 {
        self.euclidean_squared(a, b).sqrt()
    }

    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut sum_sq = 0.0;
        for i in 0..a.len() {
            let diff = a[i] - b[i];
            sum_sq += diff * diff;
        }
        sum_sq
    }
}
