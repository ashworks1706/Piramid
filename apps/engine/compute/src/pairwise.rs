//! Single-pair distance entry points; hot-path wrappers over a caller-resolved strategy. Panics on
//! mismatched lengths — a caller contract checked upstream; untrusted input should use the batch
//! kernels on [`crate::DistanceKernels`] instead.

use crate::kernels::DistanceKernels;

/// Assert the shared caller contract for all pairwise kernels.
#[inline]
fn assert_same_len(a: &[f32], b: &[f32]) {
    assert_eq!(a.len(), b.len(), "Vectors must have same length");
}

/// Cosine similarity in `[-1, 1]`; `0.0` if either operand is a zero vector.
pub fn cosine_similarity(a: &[f32], b: &[f32], kernels: &dyn DistanceKernels) -> f32 {
    assert_same_len(a, b);
    kernels.cosine(a, b)
}

/// Inner product of two vectors.
pub fn dot_product(a: &[f32], b: &[f32], kernels: &dyn DistanceKernels) -> f32 {
    assert_same_len(a, b);
    kernels.dot(a, b)
}

/// L2 distance between two vectors.
pub fn euclidean_distance(a: &[f32], b: &[f32], kernels: &dyn DistanceKernels) -> f32 {
    assert_same_len(a, b);
    kernels.euclidean(a, b)
}

/// Squared L2 distance, skipping the final `sqrt`.
///
/// Prefer this when only relative ordering matters.
pub fn euclidean_distance_squared(a: &[f32], b: &[f32], kernels: &dyn DistanceKernels) -> f32 {
    assert_same_len(a, b);
    kernels.euclidean_squared(a, b)
}
