//! Query execution: plans overfetch, asks the index for candidates, scores and filters them.

use crate::{utils::sort_and_truncate, Hit};
use piramid_compute::{strategies::for_mode, ExecutionMode, Metric};
use piramid_core::config::SearchConfig;
use piramid_core::error::{IndexError, Result};
use piramid_core::metadata::Filter;
use piramid_index::{IndexSearchRequest, MetadataReader, VectorIndex, VectorReader};
use piramid_storage::Document;
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
}

impl Default for SearchParams<'_> {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Auto,
            filter: None,
            filter_overfetch_override: None,
            search_config_override: None,
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

    // With a filter, ask for more than `k` so enough survive post-filtering.
    let base_overfetch = effective_search.filter_overfetch.max(1);
    let expansion = params
        .filter_overfetch_override
        .unwrap_or(base_overfetch)
        .max(1);
    let search_k = if params.filter.is_some() {
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
        let vec = entry.vector().to_vec();
        let score = metric.calculate(query, &vec, kernels);
        results.push(Hit {
            id,
            score,
            text: entry.text,
            vector: vec,
            metadata: entry.metadata.clone(),
        });
    }

    if let Some(filter) = params.filter {
        results.retain(|hit| filter.matches(&hit.metadata));
        sort_and_truncate(&mut results, k);
    }
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
