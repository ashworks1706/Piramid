use std::time::Instant;

use crate::runtime::SharedState;
use crate::services::search::{apply_search_overrides, hit_to_response, parse_metric};
use crate::services::types::range::RangeSearchRequest;
use crate::services::types::{
    MultiSearchResponse, SearchRequest, SearchResponse, SearchResultsResponse,
};
use piramid_core::error::{Result, ServerError};
use piramid_core::telemetry::record_lock_read;
use piramid_core::validation;

use super::{ensure_available, MAX_BATCH_SIZE};

pub fn search_vectors(
    state: &SharedState,
    collection: String,
    request_id: &str,
    req: SearchRequest,
) -> Result<SearchResultsResponse> {
    ensure_available(state)?;
    validation::validate_collection_name(&collection)?;

    let collection_handle = state.get_existing_collection(&collection)?;
    let lock_start = Instant::now();
    let collection_guard = collection_handle.read();
    record_lock_read(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    let SearchRequest {
        vector,
        vectors,
        k,
        metric,
        ef,
        nprobe,
        overfetch,
        preset,
    } = req;
    let metric = parse_metric(metric)?;
    let effective_search = apply_search_overrides(
        collection_guard.config().search,
        ef,
        nprobe,
        overfetch,
        preset,
    )?;

    match (vector, vectors) {
        (Some(vector), None) => {
            validation::validate_vector(&vector)?;
            let start = Instant::now();
            let results = collection_guard.search(
                &vector,
                k,
                metric,
                piramid_search::SearchParams {
                    mode: collection_guard.config().execution,
                    filter: None,
                    filter_overfetch_override: overfetch,
                    search_config_override: Some(effective_search),
                },
            )?;
            let duration = start.elapsed();
            if duration.as_millis() > state.slow_query_ms {
                tracing::warn!(
                    target: "piramid::search",
                    collection=%collection,
                    request_id = request_id,
                    elapsed_ms = duration.as_millis(),
                    "slow_search"
                );
            }
            if let Some(tracker) = state.collection_manager.tracker(&collection) {
                tracker.record_search(duration);
            }

            Ok(SearchResultsResponse::Single(SearchResponse {
                results: results.into_iter().map(hit_to_response).collect(),
                latency_ms: Some(duration.as_millis() as f32),
            }))
        }
        (None, Some(queries)) => {
            validation::validate_batch_size(queries.len(), MAX_BATCH_SIZE, "Search")?;
            validation::validate_vectors(&queries)?;

            let start = Instant::now();
            let params = piramid_search::SearchParams {
                mode: collection_guard.config().execution,
                filter: None,
                filter_overfetch_override: overfetch,
                search_config_override: Some(effective_search),
            };
            let batch_results = collection_guard.search_batch_with(&queries, k, metric, params)?;
            let duration = start.elapsed();
            if duration.as_millis() > state.slow_query_ms {
                tracing::warn!(
                    target: "piramid::search",
                    collection=%collection,
                    request_id = request_id,
                    elapsed_ms = duration.as_millis(),
                    "slow_batch_search"
                );
            }
            if let Some(tracker) = state.collection_manager.tracker(&collection) {
                tracker.record_search(duration);
            }

            Ok(SearchResultsResponse::Multi(MultiSearchResponse {
                results: batch_results
                    .into_iter()
                    .map(|results| results.into_iter().map(hit_to_response).collect())
                    .collect(),
                latency_ms: Some(duration.as_millis() as f32),
            }))
        }
        (Some(_), Some(_)) => Err(ServerError::InvalidRequest(
            "Provide either vector or vectors, not both".to_string(),
        )
        .into()),
        (None, None) => {
            Err(ServerError::InvalidRequest("No search vector(s) provided".to_string()).into())
        }
    }
}

pub fn range_search_vectors(
    state: &SharedState,
    collection: String,
    request_id: &str,
    req: RangeSearchRequest,
) -> Result<SearchResponse> {
    ensure_available(state)?;
    validation::validate_collection_name(&collection)?;
    validation::validate_vector(&req.vector)?;

    let collection_handle = state.get_existing_collection(&collection)?;
    let lock_start = Instant::now();
    let collection_guard = collection_handle.read();
    record_lock_read(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    let metric = parse_metric(req.metric)?;
    let effective_search = apply_search_overrides(
        collection_guard.config().search,
        req.ef,
        req.nprobe,
        req.overfetch,
        req.preset,
    )?;
    let start = Instant::now();
    let mut results = collection_guard.search(
        &req.vector,
        req.k,
        metric,
        piramid_search::SearchParams {
            mode: collection_guard.config().execution,
            filter: None,
            filter_overfetch_override: req.overfetch,
            search_config_override: Some(effective_search),
        },
    )?;
    results.retain(|hit| hit.score >= req.min_score);
    let duration = start.elapsed();
    if duration.as_millis() > state.slow_query_ms {
        tracing::warn!(
            target: "piramid::search",
            collection=%collection,
            request_id = request_id,
            elapsed_ms = duration.as_millis(),
            "slow_range_search"
        );
    }
    if let Some(tracker) = state.collection_manager.tracker(&collection) {
        tracker.record_search(duration);
    }

    Ok(SearchResponse {
        results: results.into_iter().map(hit_to_response).collect(),
        latency_ms: Some(duration.as_millis() as f32),
    })
}
