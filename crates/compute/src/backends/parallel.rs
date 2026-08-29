//! Rayon-parallel CPU backend.
//!
//! Splits operands into chunks sized to the core count. Only worth selecting for very high
//! dimensionality; for typical embedding widths the fan-out costs more than it saves, which is why
//! [`ExecutionMode::Auto`] never resolves here.

use rayon::prelude::*;

use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;

/// Multi-threaded CPU kernels.
#[derive(Debug, Default, Clone, Copy)]
pub struct ParallelBackend;

/// Chunk width that balances thread fan-out against per-chunk overhead.
fn chunk_size(len: usize) -> usize {
    (len / num_cpus::get()).max(1024)
}

impl DistanceKernels for ParallelBackend {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Parallel
    }

    fn name(&self) -> &'static str {
        "parallel"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn cosine(&self, a: &[f32], b: &[f32]) -> f32 {
        let width = chunk_size(a.len());

        let (dot, norm_a, norm_b): (f32, f32, f32) = a
            .par_chunks(width)
            .zip(b.par_chunks(width))
            .map(|(chunk_a, chunk_b)| {
                let mut dot = 0.0;
                let mut norm_a = 0.0;
                let mut norm_b = 0.0;
                for i in 0..chunk_a.len() {
                    dot += chunk_a[i] * chunk_b[i];
                    norm_a += chunk_a[i] * chunk_a[i];
                    norm_b += chunk_b[i] * chunk_b[i];
                }
                (dot, norm_a, norm_b)
            })
            .reduce(
                || (0.0, 0.0, 0.0),
                |(d1, na1, nb1), (d2, na2, nb2)| (d1 + d2, na1 + na2, nb1 + nb2),
            );

        let denominator = norm_a.sqrt() * norm_b.sqrt();
        if denominator == 0.0 {
            0.0
        } else {
            dot / denominator
        }
    }

    fn dot(&self, a: &[f32], b: &[f32]) -> f32 {
        let width = chunk_size(a.len());
        a.par_chunks(width)
            .zip(b.par_chunks(width))
            .map(|(chunk_a, chunk_b)| {
                let mut sum = 0.0;
                for i in 0..chunk_a.len() {
                    sum += chunk_a[i] * chunk_b[i];
                }
                sum
            })
            .sum()
    }

    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32 {
        self.euclidean_squared(a, b).sqrt()
    }

    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        let width = chunk_size(a.len());
        a.par_chunks(width)
            .zip(b.par_chunks(width))
            .map(|(chunk_a, chunk_b)| {
                let mut sum = 0.0;
                for i in 0..chunk_a.len() {
                    let diff = chunk_a[i] - chunk_b[i];
                    sum += diff * diff;
                }
                sum
            })
            .sum()
    }
}
