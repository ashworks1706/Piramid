//! Query execution: filtering, scoring, ranking.
//!
//! Above `index/`, below `collections/`. It can drive an index and score candidates but does not
//! know what a collection is — see [`engine::SearchTarget`].

pub mod engine;
pub mod query;
mod types;
pub mod utils;

pub use engine::{search, search_batch, SearchParams, SearchTarget};
pub use piramid_compute::Metric;
pub use query::{Filter, FilterCondition};
pub use types::Hit;
