//! Similarity metrics: a [`Metric`] is what to measure, orthogonal to which strategy measures it.

use serde::{Deserialize, Serialize};

use crate::kernels::DistanceKernels;
use crate::pairwise::{cosine_similarity, dot_product, euclidean_distance};

/// How similarity between two vectors is measured; [`Metric::calculate`] normalizes to "higher is closer".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Metric {
    /// Angle between vectors, in `[-1, 1]`.
    #[default]
    Cosine,
    /// L2 distance, mapped to `1 / (1 + d)` so higher stays better.
    Euclidean,
    /// Unnormalized inner product.
    DotProduct,
}

impl Metric {
    /// Score `a` against `b` such that a higher result always means more similar.
    pub fn calculate(&self, a: &[f32], b: &[f32], kernels: &dyn DistanceKernels) -> f32 {
        match self {
            Metric::Cosine => cosine_similarity(a, b, kernels),
            Metric::Euclidean => 1.0 / (1.0 + euclidean_distance(a, b, kernels)),
            Metric::DotProduct => dot_product(a, b, kernels),
        }
    }

    /// Stable lowercase name, used by the HTTP surface and telemetry labels.
    pub fn as_str(&self) -> &'static str {
        match self {
            Metric::Cosine => "cosine",
            Metric::Euclidean => "euclidean",
            Metric::DotProduct => "dot",
        }
    }
}
