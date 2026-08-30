#![deny(missing_docs)]

//! Distance and similarity math, and the backend dispatch that runs it.

pub mod backends;
pub mod error;
pub mod kernels;
pub mod metric;
pub mod mode;
pub mod pairwise;
pub mod quantization;

pub use error::{ComputeError, ComputeResult};
pub use kernels::{check_batch_shape, DistanceKernels};
pub use metric::Metric;
pub use mode::ExecutionMode;
pub use pairwise::{
    cosine_similarity, dot_product, euclidean_distance, euclidean_distance_squared,
};
