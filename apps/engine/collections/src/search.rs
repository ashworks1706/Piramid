//! Collection-level search: adapts collection configuration into a search target.

use piramid_compute::{ExecutionMode, Metric};
use piramid_core::Result;
use piramid_search::{Hit, SearchParams, SearchTarget};

use super::Collection;

fn target(storage: &Collection) -> SearchTarget<'_> {
    SearchTarget {
        index: storage.vector_index(),
        vectors: storage.vector_reader(),
        metadata: storage.metadata_view(),
        default_config: storage.config.search,
    }
}

/// Search one query, filling unset params from the collection's configuration.
pub fn search(
    storage: &Collection,
    query: &[f32],
    k: usize,
    metric: Metric,
    mut params: SearchParams,
) -> Result<Vec<Hit>> {
    if matches!(params.mode, ExecutionMode::Auto) {
        params.mode = storage.config().execution;
    }
    if params.filter_overfetch_override.is_none() {
        params.filter_overfetch_override = Some(storage.config.search.filter_overfetch);
    }
    piramid_search::search(&target(storage), query, k, metric, params, &|id| {
        storage.get(id)
    })
}

/// Search many queries, in parallel when the collection's parallelism config allows.
pub fn search_batch(
    storage: &Collection,
    queries: &[Vec<f32>],
    k: usize,
    metric: Metric,
    params: SearchParams,
) -> Result<Vec<Vec<Hit>>> {
    let mut params = params;
    if matches!(params.mode, ExecutionMode::Auto) {
        params.mode = storage.config().execution;
    }
    piramid_search::search_batch(
        &target(storage),
        queries,
        k,
        metric,
        params,
        storage.config().search.parallel,
        &|id| storage.get(id),
    )
}
