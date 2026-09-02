//! Query execution: filtering, scoring, ranking.

pub mod engine;
mod types;
pub mod utils;

pub use engine::{search, search_batch, SearchParams, SearchTarget};
pub use piramid_hardware::compute::Metric;
pub use types::Hit;
