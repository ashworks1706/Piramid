//! 1-bit quantized strategy: sign-bit reduction scored by Hamming agreement, a lossy pre-filter.
//!
//! Returns an approximate answer rather than the same answer faster. It serves the search
//! pre-filter path.

use crate::compute::kernels::DistanceKernels;
use crate::compute::mode::ExecutionMode;

/// Binary-quantized approximate kernels.
#[derive(Debug, Default, Clone, Copy)]
pub struct BinaryStrategy;

/// Count positions where the two operands disagree in sign.
#[inline]
fn hamming(a: &[f32], b: &[f32]) -> u32 {
    a.iter()
        .zip(b)
        .filter(|(&x, &y)| (x >= 0.0) != (y >= 0.0))
        .count() as u32
}

impl DistanceKernels for BinaryStrategy {
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
        // Map the agreement fraction from 0 to 1 onto the cosine range of -1 to 1.
        let agreement = 1.0 - (hamming(a, b) as f32 / a.len() as f32);
        2.0 * agreement - 1.0
    }

    fn dot(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b)
            .filter(|(&x, &y)| x >= 0.0 && y >= 0.0)
            .count() as f32
    }

    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32 {
        self.euclidean_squared(a, b).sqrt()
    }

    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        hamming(a, b) as f32
    }
}
