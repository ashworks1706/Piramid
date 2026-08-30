//! Explicitly vectorized CPU backend using `wide`'s `f32x8` (AVX2 on x86_64, NEON on aarch64).

use wide::f32x8;

use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;

/// SIMD CPU kernels.
#[derive(Debug, Default, Clone, Copy)]
pub struct SimdBackend;

/// Load eight contiguous lanes starting at `offset`.
#[inline(always)]
fn load(v: &[f32], offset: usize) -> f32x8 {
    f32x8::new([
        v[offset],
        v[offset + 1],
        v[offset + 2],
        v[offset + 3],
        v[offset + 4],
        v[offset + 5],
        v[offset + 6],
        v[offset + 7],
    ])
}

impl DistanceKernels for SimdBackend {
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
        let len = a.len();
        let mut dot_sum = f32x8::splat(0.0);
        let mut norm_a_sum = f32x8::splat(0.0);
        let mut norm_b_sum = f32x8::splat(0.0);

        let chunks = len / 8;
        for i in 0..chunks {
            let offset = i * 8;
            let va = load(a, offset);
            let vb = load(b, offset);
            dot_sum += va * vb;
            norm_a_sum += va * va;
            norm_b_sum += vb * vb;
        }

        let mut dot: f32 = dot_sum.to_array().iter().sum();
        let mut norm_a: f32 = norm_a_sum.to_array().iter().sum();
        let mut norm_b: f32 = norm_b_sum.to_array().iter().sum();

        for i in (chunks * 8)..len {
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
        let len = a.len();
        let mut sum = f32x8::splat(0.0);

        let chunks = len / 8;
        for i in 0..chunks {
            let offset = i * 8;
            sum += load(a, offset) * load(b, offset);
        }

        let mut result: f32 = sum.to_array().iter().sum();
        for i in (chunks * 8)..len {
            result += a[i] * b[i];
        }
        result
    }

    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32 {
        self.euclidean_squared(a, b).sqrt()
    }

    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        let len = a.len();
        let mut sum_sq = f32x8::splat(0.0);

        let chunks = len / 8;
        for i in 0..chunks {
            let offset = i * 8;
            let diff = load(a, offset) - load(b, offset);
            sum_sq += diff * diff;
        }

        let mut result: f32 = sum_sq.to_array().iter().sum();
        for i in (chunks * 8)..len {
            let diff = a[i] - b[i];
            result += diff * diff;
        }
        result
    }
}
