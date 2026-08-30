use std::time::Instant;

use crate::runtime::SharedState;
use crate::services::convert::{
    apply_search_overrides, hit_to_response, json_to_metadata, parse_filter, parse_metric,
};
use crate::services::types::*;
use crate::services::EMBEDDING_NOT_CONFIGURED;
use piramid_core::error::{Result, ServerError};
use piramid_core::metadata::Metadata;
use piramid_core::stats::{record_lock_read, record_lock_write};
use piramid_storage::Document;

fn ensure_available(state: &SharedState) -> Result<()> {
    if state
        .shutting_down
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return Err(ServerError::ServiceUnavailable("Server is shutting down".to_string()).into());
    }
    Ok(())
}

/// Embed texts through the configured provider and store the resulting vectors.
#[tracing::instrument(
    name = "embed",
    target = "piramid::embeddings",
    skip_all,
    fields(collection = %collection, texts = req.texts.len())
)]
pub async fn embed_text(
    state: &SharedState,
    collection: String,
    req: EmbedRequest,
) -> Result<EmbedResponse> {
    ensure_available(state)?;
    state.ensure_write_allowed()?;

    let EmbedRequest { texts, metadata } = req;
    if texts.is_empty() {
        return Err(ServerError::InvalidRequest("texts must not be empty".to_string()).into());
    }
    // A short list would silently leave the tail of the batch unlabelled.
    if !metadata.is_empty() && metadata.len() != texts.len() {
        return Err(ServerError::InvalidRequest(format!(
            "metadata length mismatch: {} texts, {} metadata entries",
            texts.len(),
            metadata.len()
        ))
        .into());
    }

    let collection_handle = state.get_or_create_collection(&collection)?;
    let embedder = state
        .embedder
        .as_ref()
        .ok_or(ServerError::ServiceUnavailable(
            EMBEDDING_NOT_CONFIGURED.to_string(),
        ))?;

    tracing::info!(
        target: "piramid::inference",
        collection=%collection,
        batch=texts.len(),
        "embed_request"
    );

    let mut metadata = metadata.into_iter();
    let mut embeddings = Vec::with_capacity(texts.len());
    let mut entries = Vec::with_capacity(texts.len());
    let mut total_tokens: u32 = 0;
    let start = Instant::now();
    for text in texts {
        let response = embedder.embed(&text).await?;
        embeddings.push(response.embedding.clone());
        if let Some(tokens) = response.tokens {
            total_tokens = total_tokens.saturating_add(tokens);
        }
        let metadata = match metadata.next() {
            Some(map) => json_to_metadata(map)?,
            None => Metadata::new(),
        };
        entries.push(Document::with_metadata(response.embedding, text, metadata));
    }

    let lock_start = Instant::now();
    let mut collection_guard = collection_handle.write();
    record_lock_write(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    let ids = collection_guard.insert_batch(entries)?;
    state.enforce_cache_budget();
    state
        .embed_metrics
        .record(1, ids.len() as u64, total_tokens as u64, start.elapsed());

    Ok(EmbedResponse {
        ids: ids.into_iter().map(|id| id.to_string()).collect(),
        embeddings,
        total_tokens: (total_tokens > 0).then_some(total_tokens),
    })
}

/// Embed a query string, then search with the resulting vector.
#[tracing::instrument(
    name = "search_by_text",
    target = "piramid::search",
    skip_all,
    fields(collection = %collection, request_id = request_id)
)]
pub async fn search_by_text(
    state: &SharedState,
    collection: String,
    request_id: &str,
    req: TextSearchRequest,
) -> Result<SearchResponse> {
    ensure_available(state)?;

    let collection_handle = state.get_existing_collection(&collection)?;
    let embedder = state
        .embedder
        .as_ref()
        .ok_or(ServerError::ServiceUnavailable(
            EMBEDDING_NOT_CONFIGURED.to_string(),
        ))?;

    tracing::info!(
        target: "piramid::search",
        collection=%collection,
        "search_by_text_request"
    );
    let start = Instant::now();
    let response = embedder.embed(&req.query).await?;
    let embed_duration = start.elapsed();
    state
        .embed_metrics
        .record(1, 1, response.tokens.unwrap_or(0) as u64, embed_duration);

    let metric = parse_metric(req.metric)?;
    let filter = parse_filter(req.filter)?;
    let base_search = {
        let collection_guard = collection_handle.read();
        collection_guard.config().search
    };
    let effective_search = apply_search_overrides(base_search, &req.tuning)?;

    let lock_start = Instant::now();
    let collection_guard = collection_handle.read();
    record_lock_read(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    let start = Instant::now();
    let results = collection_guard.search(
        &response.embedding,
        req.k,
        metric,
        piramid_search::SearchParams {
            mode: collection_guard.config().execution,
            filter: filter.as_ref(),
            filter_overfetch_override: req.tuning.overfetch,
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
            "slow_text_search"
        );
    }
    if let Some(tracker) = state.collection_manager.tracker(&collection) {
        tracker.record_search(duration);
    }

    Ok(SearchResponse {
        results: vec![results.into_iter().map(hit_to_response).collect()],
        latency_ms: duration.as_millis() as f32,
    })
}
