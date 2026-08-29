//! Distance and similarity math, and the backend dispatch that runs it.
//!
//! # Layout
//!
//! - [`metric`] — *what* to measure ([`Metric`]).
//! - [`mode`] — *where* to measure it ([`ExecutionMode`]).
//! - [`kernels`] — the [`DistanceKernels`] contract every backend implements.
//! - [`backends`] — one file per backend, plus the registry that maps a mode to one.
//! - [`pairwise`] — single-pair convenience wrappers over the selected backend.
//!
//! # Layering
//!
//! This module is a leaf: it depends on nothing else in the crate. In particular it does **not**
//! depend on `config/` — [`ExecutionMode`] lives here and is re-exported by `config/` for
//! callers, not the other way around. Keeping it a leaf is what allows kernels to be benchmarked
//! and swapped without dragging application state along.
//!
//! Device runtime concerns — contexts, buffers, streams — belong in [`piramid_gpu`], not here.
//! This module owns math semantics and backend *selection* only.

pub mod backends;
pub mod error;
pub mod kernels;
pub mod metric;
pub mod mode;
pub mod pairwise;

pub use error::{ComputeError, ComputeResult};
pub use kernels::{check_batch_shape, check_dims, DistanceKernels};
pub use metric::Metric;
pub use mode::ExecutionMode;
pub use pairwise::{
    cosine_similarity, dot_product, euclidean_distance, euclidean_distance_squared,
};
