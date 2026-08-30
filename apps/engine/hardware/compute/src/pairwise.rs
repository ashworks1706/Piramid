//! Single-pair distance entry points.
//!
//! Hot-path wrappers for index traversal and reranking. They resolve a backend through
//! [`crate::backends::resolve_available`], so an unavailable one degrades to CPU.
//!
//! # Panics
//!
//! Every function asserts both operands have the same length. That is a caller contract:
//! dimensions are checked at the collection boundary, so a mismatch here is a bug upstream. For
//! untrusted input use the batch kernels on [`crate::DistanceKernels`], which return
//! [`ComputeError::ShapeMismatch`](crate::ComputeError::ShapeMismatch) instead.

use crate::backends::resolve_available;
use crate::mode::ExecutionMode;

/// Assert the shared caller contract for all pairwise kernels.
#[inline]
fn assert_same_len(a: &[f32], b: &[f32]) {
    assert_eq!(a.len(), b.len(), "Vectors must have same length");
}

/// Cosine similarity in `[-1, 1]`; `0.0` if either operand is a zero vector.
pub fn cosine_similarity(a: &[f32], b: &[f32], mode: ExecutionMode) -> f32 {
    assert_same_len(a, b);
    resolve_available(mode).cosine(a, b)
}

/// Inner product of two vectors.
pub fn dot_product(a: &[f32], b: &[f32], mode: ExecutionMode) -> f32 {
    assert_same_len(a, b);
    resolve_available(mode).dot(a, b)
}

/// L2 distance between two vectors.
pub fn euclidean_distance(a: &[f32], b: &[f32], mode: ExecutionMode) -> f32 {
    assert_same_len(a, b);
    resolve_available(mode).euclidean(a, b)
}

/// Squared L2 distance, skipping the final `sqrt`.
///
/// Prefer this when only relative ordering matters.
pub fn euclidean_distance_squared(a: &[f32], b: &[f32], mode: ExecutionMode) -> f32 {
    assert_same_len(a, b);
    resolve_available(mode).euclidean_squared(a, b)
}
