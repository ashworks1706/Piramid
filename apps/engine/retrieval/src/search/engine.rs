//! Query execution: plans overfetch, asks the index for candidates, scores and filters them.

use crate::index::{IndexSearchRequest, MetadataReader, VectorIndex, VectorReader};
use piramid_core::config::SearchConfig;
use piramid_core::error::{IndexError, Result};
use piramid_core::metadata::Filter;
use piramid_core::Document;
use piramid_core::Hit;
use piramid_hardware::compute::{strategies::for_mode, ExecutionMode, Metric};
use uuid::Uuid;

/// Per-query overrides layered on top of a collection's configured defaults.
#[derive(Debug, Clone, Copy)]
pub struct SearchParams<'a> {
    /// Backend to score with. `Auto` defers to the collection's configured mode.
    pub mode: ExecutionMode,
    /// Metadata predicate. When present, the planner overfetches and post-filters.
    pub filter: Option<&'a Filter>,
    /// Multiplier applied to `k` when a filter is present, overriding the configured value.
    pub filter_overfetch_override: Option<usize>,
    /// Recall/speed knobs for this query, overriding the configured value.
    pub search_config_override: Option<SearchConfig>,
    /// Drop hits scoring below this. Asks a range question rather than a top-k one, so it must be
    /// applied before `k` truncates — otherwise a qualifying document loses its place to a
    /// higher-ranked one that does not qualify, and the caller sees fewer results than exist.
    pub min_score: Option<f32>,
}

impl Default for SearchParams<'_> {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Auto,
            filter: None,
            filter_overfetch_override: None,
            search_config_override: None,
            min_score: None,
        }
    }
}

/// What a search runs against. Borrowed views only; the caller owns everything.
pub struct SearchTarget<'a> {
    /// The ANN index to query.
    pub index: &'a dyn VectorIndex,
    /// Access to the vectors the index refers to.
    pub vectors: &'a dyn VectorReader,
    /// Access to per-document metadata, for filter evaluation.
    pub metadata: &'a dyn MetadataReader,
    /// Recall/speed defaults, used unless [`SearchParams::search_config_override`] is set.
    pub default_config: SearchConfig,
}

/// Run one query against `target`.
pub fn search(
    target: &SearchTarget<'_>,
    query: &[f32],
    k: usize,
    metric: Metric,
    params: SearchParams<'_>,
    resolve: &(dyn Fn(&Uuid) -> Result<Option<Document>> + Sync),
) -> Result<Vec<Hit>> {
    let effective_search = params
        .search_config_override
        .unwrap_or(target.default_config);

    // Anything applied after the index returns is a post-filter, so ask for more than `k` and
    // let the survivors compete. A score threshold narrows the set exactly like a metadata
    // predicate does.
    let post_filtered = params.filter.is_some() || params.min_score.is_some();
    let base_overfetch = effective_search.filter_overfetch.max(1);
    let expansion = params
        .filter_overfetch_override
        .unwrap_or(base_overfetch)
        .max(1);
    let search_k = if post_filtered {
        k.saturating_mul(expansion)
    } else {
        k
    };

    let kernels = for_mode(params.mode)?;
    let neighbor_ids = target.index.search(
        IndexSearchRequest::new(
            query,
            search_k,
            target.vectors,
            effective_search,
            target.metadata,
        )
        .with_filter(params.filter),
    )?;

    let mut results = Vec::with_capacity(neighbor_ids.len());
    for id in neighbor_ids {
        let entry = resolve(&id)?.ok_or_else(|| {
            IndexError::SearchFailed(format!("index returned missing document {id}"))
        })?;
        let score = metric.calculate(query, entry.vector(), kernels);
        results.push(Hit {
            score,
            document: entry,
        });
    }

    if let Some(min_score) = params.min_score {
        results.retain(|hit| hit.score >= min_score);
    }
    if let Some(filter) = params.filter {
        results.retain(|hit| filter.matches(&hit.document.metadata));
    }
    // Unconditional: the index orders by its own traversal, and `score` is recomputed here.
    rank_top_k(&mut results, k);
    Ok(results)
}

/// Run several queries against `target`, optionally in parallel.
pub fn search_batch(
    target: &SearchTarget<'_>,
    queries: &[Vec<f32>],
    k: usize,
    metric: Metric,
    params: SearchParams<'_>,
    parallel: bool,
    resolve: &(dyn Fn(&Uuid) -> Result<Option<Document>> + Sync),
) -> Result<Vec<Vec<Hit>>> {
    if parallel {
        use rayon::prelude::*;
        queries
            .par_iter()
            .map(|query| search(target, query, k, metric, params, resolve))
            .collect()
    } else {
        queries
            .iter()
            .map(|query| search(target, query, k, metric, params, resolve))
            .collect()
    }
}

/// Sort by score descending and keep the top `k`.
fn rank_top_k(results: &mut Vec<Hit>, k: usize) {
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(k);
}
