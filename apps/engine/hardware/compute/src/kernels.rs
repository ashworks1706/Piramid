//! The kernel contract every execution strategy implements; batch methods take a row-major slab.

use crate::error::{ComputeError, ComputeResult};
use crate::mode::ExecutionMode;

/// Distance and similarity kernels for one execution strategy.
pub trait DistanceKernels: Send + Sync {
    /// The execution mode this strategy serves.
    fn mode(&self) -> ExecutionMode;

    /// Stable name for logs, metrics, and error messages.
    fn name(&self) -> &'static str;

    /// Whether this strategy can actually run on this machine right now.
    fn is_available(&self) -> bool;

    // Pairwise. Callers guarantee equal lengths.

    /// Cosine similarity of two equal-length vectors.
    fn cosine(&self, a: &[f32], b: &[f32]) -> f32;

    /// Inner product of two equal-length vectors.
    fn dot(&self, a: &[f32], b: &[f32]) -> f32;

    /// L2 distance between two equal-length vectors.
    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32;

    /// Squared L2 distance, skipping the final `sqrt`.
    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32;

    // Batch. Defaults loop over pairwise, which is right for CPU; devices override with a launch.

    /// Score `query` against every row of the row-major `candidates` slab.
    fn cosine_batch(
        &self,
        query: &[f32],
        candidates: &[f32],
        dim: usize,
        out: &mut [f32],
    ) -> ComputeResult<()> {
        batch_via_pairwise(query, candidates, dim, out, |a, b| self.cosine(a, b))
    }

    /// Inner product of `query` against every row of the `candidates` slab.
    fn dot_batch(
        &self,
        query: &[f32],
        candidates: &[f32],
        dim: usize,
        out: &mut [f32],
    ) -> ComputeResult<()> {
        batch_via_pairwise(query, candidates, dim, out, |a, b| self.dot(a, b))
    }

    /// L2 distance from `query` to every row of the `candidates` slab.
    fn euclidean_batch(
        &self,
        query: &[f32],
        candidates: &[f32],
        dim: usize,
        out: &mut [f32],
    ) -> ComputeResult<()> {
        batch_via_pairwise(query, candidates, dim, out, |a, b| self.euclidean(a, b))
    }
}

/// Validate the slab/`out` shape shared by every `*_batch` kernel; returns the row count.
pub fn check_batch_shape(
    query: &[f32],
    candidates: &[f32],
    dim: usize,
    out: &[f32],
) -> ComputeResult<usize> {
    if dim == 0 {
        return Err(ComputeError::ShapeMismatch {
            expected: 1,
            got: 0,
        });
    }
    if query.len() != dim {
        return Err(ComputeError::ShapeMismatch {
            expected: dim,
            got: query.len(),
        });
    }
    if !candidates.len().is_multiple_of(dim) {
        return Err(ComputeError::ShapeMismatch {
            expected: candidates.len().next_multiple_of(dim),
            got: candidates.len(),
        });
    }
    let rows = candidates.len() / dim;
    if out.len() != rows {
        return Err(ComputeError::ShapeMismatch {
            expected: rows,
            got: out.len(),
        });
    }
    Ok(rows)
}

/// Shared default body for the batch kernels: validate, then fold over rows.
fn batch_via_pairwise(
    query: &[f32],
    candidates: &[f32],
    dim: usize,
    out: &mut [f32],
    score: impl Fn(&[f32], &[f32]) -> f32,
) -> ComputeResult<()> {
    check_batch_shape(query, candidates, dim, out)?;
    for (row, slot) in candidates.chunks_exact(dim).zip(out.iter_mut()) {
        *slot = score(query, row);
    }
    Ok(())
}
