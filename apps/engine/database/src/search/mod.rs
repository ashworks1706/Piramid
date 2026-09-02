//! Query execution: filtering, scoring, ranking.

pub mod engine;
pub mod near_duplicates;

pub use engine::{search, search_batch, SearchParams, SearchTarget};
pub use near_duplicates::{near_duplicates, DuplicatePair, DuplicateParams};
