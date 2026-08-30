//! 1-bit quantized strategy: sign-bit reduction scored by Hamming agreement, a lossy pre-filter.
//!
//! Not an execution strategy like its siblings — it returns a *different, approximate* answer
//! rather than the same one faster. See ADR 0013; it belongs in the search pre-filter path.

use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;

/// Binary-quantized approximate kernels.
#[derive(Debug, Default, Clone, Copy)]
pub struct BinaryBackend;

/// Count positions where the two operands disagree in sign.
#[inline]
fn hamming(a: &[f32], b: &[f32]) -> u32 {
    let mut distance = 0u32;
    for i in 0..a.len() {
        if (a[i] >= 0.0) != (b[i] >= 0.0) {
            distance += 1;
        }
    }
    distance
}

impl DistanceKernels for BinaryBackend {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Binary
    }

    fn name(&self) -> &'static str {
        "binary"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn cosine(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.is_empty() {
            return 0.0;
        }
        // Map agreement fraction from [0, 1] onto cosine's [-1, 1] range.
        let agreement = 1.0 - (hamming(a, b) as f32 / a.len() as f32);
        2.0 * agreement - 1.0
    }

    fn dot(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut both_positive = 0u32;
        for i in 0..a.len() {
            if a[i] >= 0.0 && b[i] >= 0.0 {
                both_positive += 1;
            }
        }
        both_positive as f32
    }

    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32 {
        self.euclidean_squared(a, b).sqrt()
    }

    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        hamming(a, b) as f32
    }
}
