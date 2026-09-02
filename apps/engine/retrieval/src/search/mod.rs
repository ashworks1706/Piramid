//! Query execution: filtering, scoring, ranking.

pub mod engine;

pub use engine::{search, search_batch, SearchParams, SearchTarget};
