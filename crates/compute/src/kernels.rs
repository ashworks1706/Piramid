//! The kernel contract every compute backend implements.
//!
//! # Adding a backend
//!
//! Implement [`DistanceKernels`] in a single new file under [`crate::backends`], add one
//! arm to [`crate::backends::for_mode`], and you are done. No other file in the crate
//! changes. That is the whole point of this trait.
//!
//! # Why the batch methods look like that
//!
//! [`DistanceKernels::cosine_batch`] and friends take candidates as one **contiguous, row-major
//! `&[f32]` slab** with an explicit `dim`, and write into a caller-owned `out: &mut [f32]`.
//!
//! This shape is deliberate and non-negotiable for new kernels:
//!
//! - A slab can be uploaded to a device in one `memcpy`. A `&[Vec<f32>]` cannot: each row is a
//!   separate allocation, so a device backend would have to gather every call, which costs more
//!   than the kernel saves.
//! - A caller-owned `out` keeps allocation out of the kernel, so the same buffer can be reused
//!   across queries and, later, pinned for async device transfer.
//!
//! CPU backends get correct batch behavior for free from the default implementations below.
//! A device backend overrides them with a real launch.

use crate::error::{ComputeError, ComputeResult};
use crate::mode::ExecutionMode;

/// Distance and similarity kernels for one execution backend.
///
/// Implementors must be cheap to construct and safe to share across threads; they hold backend
/// handles, never per-query state.
pub trait DistanceKernels: Send + Sync {
    /// The execution mode this backend serves.
    fn mode(&self) -> ExecutionMode;

    /// Stable name for logs, metrics, and error messages.
    fn name(&self) -> &'static str;

    /// Whether this backend can actually run on this machine right now.
    ///
    /// Compiled-out or hardware-absent backends return `false` and are skipped by
    /// [`crate::backends::resolve_available`].
    fn is_available(&self) -> bool;

    // ---- pairwise ----------------------------------------------------------------------------
    // Hot-path, single-pair kernels. Callers guarantee equal lengths; use `check_dims` first if
    // the input is untrusted.

    /// Cosine similarity of two equal-length vectors.
    fn cosine(&self, a: &[f32], b: &[f32]) -> f32;

    /// Inner product of two equal-length vectors.
    fn dot(&self, a: &[f32], b: &[f32]) -> f32;

    /// L2 distance between two equal-length vectors.
    fn euclidean(&self, a: &[f32], b: &[f32]) -> f32;

    /// Squared L2 distance, skipping the final `sqrt`.
    fn euclidean_squared(&self, a: &[f32], b: &[f32]) -> f32;

    // ---- batch -------------------------------------------------------------------------------
    // Device-shaped kernels. Defaults loop over the pairwise methods, which is correct for every
    // CPU backend; device backends override with a single launch.

    /// Score `query` against every row of the `candidates` slab.
    ///
    /// `candidates` is row-major with `candidates.len() / dim` rows. `out` must have exactly one
    /// slot per row.
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

/// Validate that two operands agree in length.
///
/// Kernels called with untrusted input should route through this instead of asserting.
pub fn check_dims(a: &[f32], b: &[f32]) -> ComputeResult<()> {
    if a.len() == b.len() {
        Ok(())
    } else {
        Err(ComputeError::ShapeMismatch {
            expected: a.len(),
            got: b.len(),
        })
    }
}

/// Validate the slab/`out` shape shared by every `*_batch` kernel.
///
/// Returns the number of candidate rows.
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
