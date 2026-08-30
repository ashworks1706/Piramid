//! Collection-level search: adapts collection configuration into a search target.

use piramid_compute::{ExecutionMode, Metric};
use piramid_core::Result;
use piramid_search::{Hit, SearchParams, SearchTarget};

use super::Collection;

fn target(collection: &Collection) -> SearchTarget<'_> {
    SearchTarget {
        index: collection.vector_index(),
        vectors: collection.vector_reader(),
        metadata: collection.metadata_view(),
        default_config: collection.config.search,
    }
}

/// Search one query, filling unset params from the collection's configuration.
pub fn search(
    collection: &Collection,
    query: &[f32],
    k: usize,
    metric: Metric,
    mut params: SearchParams,
) -> Result<Vec<Hit>> {
    if matches!(params.mode, ExecutionMode::Auto) {
        params.mode = collection.config().execution;
    }
    if params.filter_overfetch_override.is_none() {
        params.filter_overfetch_override = Some(collection.config.search.filter_overfetch);
    }
    piramid_search::search(&target(collection), query, k, metric, params, &|id| {
        collection.get(id)
    })
}

/// Search many queries, in parallel when the collection's parallelism config allows.
pub fn search_batch(
    collection: &Collection,
    queries: &[Vec<f32>],
    k: usize,
    metric: Metric,
    params: SearchParams,
) -> Result<Vec<Vec<Hit>>> {
    let mut params = params;
    if matches!(params.mode, ExecutionMode::Auto) {
        params.mode = collection.config().execution;
    }
    piramid_search::search_batch(
        &target(collection),
        queries,
        k,
        metric,
        params,
        collection.config().parallelism.parallel_search,
        &|id| collection.get(id),
    )
}
