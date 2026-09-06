//! Explicitly vectorized CPU strategy using the f32x8 type from wide (AVX2 on x86_64, NEON on
//! aarch64).

use wide::f32x8;

use crate::compute::kernels::DistanceKernels;
use crate::compute::mode::ExecutionMode;

/// SIMD CPU kernels.
#[derive(Debug, Default, Clone, Copy)]
pub struct SimdStrategy;

/// Load an exact 8-element chunk into a lane vector.
///
/// Callers pass a chunk from chunks_exact(8); a shorter chunk panics on the index.
#[inline(always)]
fn load(chunk: &[f32]) -> f32x8 {
    f32x8::new([
        chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
    ])
}

impl DistanceKernels for SimdStrategy {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Simd
    }

    fn name(&self) -> &'static str {
        "simd"
    }

    fn is_available(&self) -> bool {
        cfg!(any(target_arch = "x86_64", target_arch = "aarch64"))
    }

    fn cosine(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut dot_sum = f32x8::splat(0.0);
        let mut norm_a_sum = f32x8::splat(0.0);
        let mut norm_b_sum = f32x8::splat(0.0);

        let a_chunks = a.chunks_exact(8);
        let b_chunks = b.chunks_exact(8);
        let a_rem = a_chunks.remainder();
        let b_rem = b_chunks.remainder();

        for (ca, cb) in a_chunks.zip(b_chunks) {
            let va = load(ca);
            let vb = load(cb);
            dot_sum += va * vb;
            norm_a_sum += va * va;
            norm_b_sum += vb * vb;
        }

        let mut dot: f32 = dot_sum.to_array().iter().sum();
        let mut norm_a: f32 = norm_a_sum.to_array().iter().sum();
        let mut norm_b: f32 = norm_b_sum.to_array().iter().sum();

        for (&x, &y) in a_rem.iter().zip(b_rem) {
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
        let mut sum = f32x8::splat(0.0);

        let a_chunks = a.chunks_exact(8);
        let b_chunks = b.chunks_exact(8);
        let a_rem = a_chunks.remainder();
        let b_rem = b_chunks.remainder();

        for (ca, cb) in a_chunks.zip(b_chunks) {
            sum += load(ca) * load(cb);
        }

        let mut result: f32 = sum.to_array().iter().sum();
        for (&x, &y) in a_rem.iter().zip(b_rem) {
            result += x * y;
        }
        result
    }

    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32 {
        self.euclidean_squared(a, b).sqrt()
    }

    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut sum_sq = f32x8::splat(0.0);

        let a_chunks = a.chunks_exact(8);
        let b_chunks = b.chunks_exact(8);
        let a_rem = a_chunks.remainder();
        let b_rem = b_chunks.remainder();

        for (ca, cb) in a_chunks.zip(b_chunks) {
            let diff = load(ca) - load(cb);
            sum_sq += diff * diff;
        }

        let mut result: f32 = sum_sq.to_array().iter().sum();
        for (&x, &y) in a_rem.iter().zip(b_rem) {
            let diff = x - y;
            result += diff * diff;
        }
        result
    }
}
