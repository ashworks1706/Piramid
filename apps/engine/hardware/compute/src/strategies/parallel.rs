//! Rayon-parallel CPU strategy; chunks operands across cores, worthwhile only at high dimensionality.

use rayon::prelude::*;

use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;

/// Multi-threaded CPU kernels.
#[derive(Debug, Default, Clone, Copy)]
pub struct ParallelStrategy;

/// Chunk width that balances thread fan-out against per-chunk overhead.
fn chunk_size(len: usize) -> usize {
    (len / num_cpus::get()).max(1024)
}

impl DistanceKernels for ParallelStrategy {
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
                for (&x, &y) in chunk_a.iter().zip(chunk_b) {
                    dot += x * y;
                    norm_a += x * x;
                    norm_b += y * y;
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
                for (&x, &y) in chunk_a.iter().zip(chunk_b) {
                    sum += x * y;
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
                for (&x, &y) in chunk_a.iter().zip(chunk_b) {
                    let diff = x - y;
                    sum += diff * diff;
                }
                sum
            })
            .sum()
    }
}
