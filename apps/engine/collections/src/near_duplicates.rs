//! Collection-level duplicate scan: adapts collection configuration into a search target.

use piramid_core::error::Result;
use piramid_hardware::compute::Metric;
use piramid_retrieval::search::{DuplicatePair, DuplicateParams};

use super::Collection;

/// Neighbours examined per document when the caller does not say.
///
/// Includes each document's own hit, so this compares against 48 others.
const DEFAULT_NEIGHBORS: usize = 49;

/// Scan `collection` for near-identical pairs, filling unset knobs from its configuration.
#[allow(clippy::too_many_arguments)]
pub fn find_duplicates(
    collection: &Collection,
    metric: Metric,
    threshold: f32,
    limit: Option<usize>,
    neighbors_override: Option<usize>,
    ef_override: Option<usize>,
    nprobe_override: Option<usize>,
) -> Result<Vec<DuplicatePair>> {
    let mut search = collection.config.search;
    if let Some(ef) = ef_override {
        search.ef = Some(ef);
    }
    if let Some(nprobe) = nprobe_override {
        search.nprobe = Some(nprobe);
    }

    piramid_retrieval::search::near_duplicates(
        &super::search_target::target(collection),
        metric,
        collection.config().execution,
        DuplicateParams {
            threshold,
            neighbors: neighbors_override.unwrap_or(DEFAULT_NEIGHBORS),
            limit,
            search_config_override: Some(search),
        },
    )
}
