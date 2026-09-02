//! Ranking helpers.

use crate::search::Hit;

/// Sort by score descending and keep the top `k`.
pub fn sort_and_truncate(results: &mut Vec<Hit>, k: usize) {
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(k);
}
