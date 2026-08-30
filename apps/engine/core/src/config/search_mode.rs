//! Which kind of search a request is asking for.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SearchMode {
    /// Return the `k` nearest.
    #[default]
    KNN,
    /// Return everything within a score threshold.
    Range,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RangeSearchParams {
    pub max_distance: f32,

    /// Result ceiling. `None` returns everything above the threshold.
    pub max_results: Option<usize>,
}

impl RangeSearchParams {
    pub fn new(max_distance: f32) -> Self {
        RangeSearchParams {
            max_distance,
            max_results: None,
        }
    }
}
