//! Single-pair distance entry points.
//!
//! These are the hot-path convenience wrappers used by index traversal and reranking. They select
//! a backend via [`crate::backends::resolve_available`], so an unavailable backend
//! degrades to CPU rather than failing the query.
//!
//! # Panics
//!
//! Every function here asserts that both operands have the same length. That is a caller
//! contract, not a runtime condition — mismatched dimensions mean a bug upstream of the kernel,
//! and vectors are dimension-checked at the collection boundary long before they reach here.
//! Validated once at this layer so the backends can assume well-formed input.
//!
//! For untrusted input, or when a mismatch should be recoverable, use the batch kernels on
//! [`crate::DistanceKernels`], which return
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
