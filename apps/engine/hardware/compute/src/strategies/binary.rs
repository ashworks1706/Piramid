//! 1-bit quantized strategy: sign-bit reduction scored by Hamming agreement, a lossy pre-filter.
//!
//! Not an execution strategy like its siblings — it returns a *different, approximate* answer
//! rather than the same one faster. See ADR 0013; it belongs in the search pre-filter path.

use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;

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
        // Map agreement fraction from [0, 1] onto cosine's [-1, 1] range.
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
