use std::collections::HashMap;
use std::time::Instant;

use uuid::Uuid;

use crate::runtime::SharedState;
use crate::services::metadata::json_to_metadata;
use crate::services::types::{
    DeleteResponse, DeleteResultsResponse, DeleteVectorsRequest, InsertRequest, InsertResponse,
    InsertResultsResponse, MultiDeleteResponse, MultiInsertResponse, UpsertRequest, UpsertResponse,
};
use piramid_core::error::{Result, ServerError};
use piramid_core::telemetry::record_lock_write;
use piramid_core::validation;
use piramid_storage::Document;

use super::{ensure_available, MAX_BATCH_SIZE};

fn build_single_entry(mut req: InsertRequest) -> Result<Document> {
    let text = req.text.clone().ok_or_else(|| {
        ServerError::InvalidRequest("text is required for single insert".to_string())
    })?;
    validation::validate_text(&text)?;
    let vector = req.vector.take().ok_or_else(|| {
        ServerError::InvalidRequest("vector is required for single insert".to_string())
    })?;
    validation::validate_vector(&vector)?;
    let vector = if req.normalize {
        validation::normalize_vector(&vector)
    } else {
        vector
    };
    Ok(Document::with_metadata(
        vector,
        text,
        json_to_metadata(req.metadata),
    ))
}

fn build_batch_entries(mut req: InsertRequest) -> Result<Vec<Document>> {
    let vectors = req.vectors.take().ok_or_else(|| {
        ServerError::InvalidRequest("vectors are required for batch insert".to_string())
    })?;
    let texts = req.texts.clone().ok_or_else(|| {
        ServerError::InvalidRequest("texts are required for batch insert".to_string())
    })?;
    validation::validate_batch_size(vectors.len(), MAX_BATCH_SIZE, "Insert")?;
    if vectors.len() != texts.len() {
        return Err(
            ServerError::InvalidRequest("vectors and texts length mismatch".to_string()).into(),
        );
    }
    validation::validate_vectors(&vectors)?;
    for text in &texts {
        validation::validate_text(text)?;
    }

    let vectors = if req.normalize {
        vectors
            .iter()
            .map(|vector| validation::normalize_vector(vector))
            .collect()
    } else {
        vectors
    };

    let mut entries = Vec::with_capacity(vectors.len());
    for (idx, vector) in vectors.into_iter().enumerate() {
        let metadata = if idx < req.metadata_list.len() {
            json_to_metadata(req.metadata_list[idx].clone())
        } else {
            json_to_metadata(HashMap::new())
        };
        entries.push(Document::with_metadata(
            vector,
            texts[idx].clone(),
            metadata,
        ));
    }
    Ok(entries)
}

pub fn insert_vector(
    state: &SharedState,
    collection: String,
    mut req: InsertRequest,
) -> Result<InsertResultsResponse> {
    ensure_available(state)?;
    state.ensure_write_allowed()?;
    validation::validate_collection_name(&collection)?;

    let collection_handle = state.get_or_create_collection(&collection)?;
    tracing::info!(
        target: "piramid::writes",
        collection=%collection,
        single=req.vector.is_some(),
        batch=req.vectors.as_ref().map(|vectors| vectors.len()),
        "insert_request"
    );

    let lock_start = Instant::now();
    let mut collection_guard = collection_handle.write();
    record_lock_write(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    let response = match (req.vector.take(), req.vectors.take()) {
        (Some(vector), None) => {
            req.vector = Some(vector);
            let entry = build_single_entry(req)?;
            let start = Instant::now();
            let id = collection_guard.insert(entry)?;
            let duration = start.elapsed();

            if let Some(tracker) = state.collection_manager.tracker(&collection) {
                tracker.record_insert(duration);
            }
            state.enforce_cache_budget();

            InsertResultsResponse::Single(InsertResponse {
                id: id.to_string(),
                latency_ms: Some(duration.as_millis() as f32),
            })
        }
        (None, Some(vectors)) => {
            req.vectors = Some(vectors);
            let count = req.texts.as_ref().map(|texts| texts.len()).unwrap_or(0);
            let entries = build_batch_entries(req)?;
            let start = Instant::now();
            let ids = collection_guard.insert_batch(entries)?;
            let duration = start.elapsed();

            if let Some(tracker) = state.collection_manager.tracker(&collection) {
                tracker.record_insert(duration);
            }
            state.enforce_cache_budget();

            InsertResultsResponse::Multi(MultiInsertResponse {
                ids: ids.into_iter().map(|id| id.to_string()).collect(),
                count,
                latency_ms: Some(duration.as_millis() as f32),
            })
        }
        (Some(_), Some(_)) => {
            return Err(ServerError::InvalidRequest(
                "Provide either vector or vectors, not both".to_string(),
            )
            .into())
        }
        (None, None) => {
            return Err(ServerError::InvalidRequest("No vectors provided".to_string()).into())
        }
    };

    Ok(response)
}

pub fn delete_vector(
    state: &SharedState,
    collection: String,
    id: String,
) -> Result<DeleteResultsResponse> {
    ensure_available(state)?;
    state.ensure_write_allowed()?;

    let collection_handle = state.get_existing_collection(&collection)?;
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ServerError::InvalidRequest("Invalid UUID".to_string()))?;

    let lock_start = Instant::now();
    let mut collection_guard = collection_handle.write();
    record_lock_write(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    let start = Instant::now();
    let deleted = collection_guard.delete(&uuid)?;
    let duration = start.elapsed();

    if let Some(tracker) = state.collection_manager.tracker(&collection) {
        tracker.record_delete(duration);
    }

    Ok(DeleteResultsResponse::Single(DeleteResponse {
        deleted,
        latency_ms: Some(duration.as_millis() as f32),
    }))
}

pub fn delete_vectors(
    state: &SharedState,
    collection: String,
    req: DeleteVectorsRequest,
) -> Result<DeleteResultsResponse> {
    ensure_available(state)?;
    state.ensure_write_allowed()?;
    validation::validate_collection_name(&collection)?;
    validation::validate_batch_size(req.ids.len(), MAX_BATCH_SIZE, "Delete")?;

    let collection_handle = state.get_existing_collection(&collection)?;
    let mut uuids = Vec::with_capacity(req.ids.len());
    for id in &req.ids {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| ServerError::InvalidRequest(format!("Invalid UUID: {}", id)))?;
        uuids.push(uuid);
    }

    let lock_start = Instant::now();
    let mut collection_guard = collection_handle.write();
    record_lock_write(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    let start = Instant::now();
    let deleted_count = collection_guard.delete_batch(&uuids)?;
    let duration = start.elapsed();

    if let Some(tracker) = state.collection_manager.tracker(&collection) {
        tracker.record_delete(duration);
    }

    Ok(DeleteResultsResponse::Multi(MultiDeleteResponse {
        deleted_count,
        latency_ms: Some(duration.as_millis() as f32),
    }))
}

pub fn upsert_vector(
    state: &SharedState,
    collection: String,
    mut req: UpsertRequest,
) -> Result<UpsertResponse> {
    ensure_available(state)?;
    state.ensure_write_allowed()?;
    validation::validate_collection_name(&collection)?;
    validation::validate_text(&req.text)?;
    validation::validate_vector(&req.vector)?;

    if req.normalize {
        req.vector = validation::normalize_vector(&req.vector);
    }

    let collection_handle = state.get_or_create_collection(&collection)?;
    let lock_start = Instant::now();
    let mut collection_guard = collection_handle.write();
    record_lock_write(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    let id = if let Some(id) = req.id {
        Uuid::parse_str(&id).map_err(|_| ServerError::InvalidRequest("Invalid UUID".to_string()))?
    } else {
        Uuid::new_v4()
    };
    let exists = collection_guard.get(&id)?.is_some();
    let mut entry = Document::with_metadata(req.vector, req.text, json_to_metadata(req.metadata));
    entry.id = id;

    let start = Instant::now();
    collection_guard.upsert(entry)?;
    let duration = start.elapsed();

    if let Some(tracker) = state.collection_manager.tracker(&collection) {
        if exists {
            tracker.record_update(duration);
        } else {
            tracker.record_insert(duration);
        }
    }
    state.enforce_cache_budget();
    tracing::info!(
        target: "piramid::writes",
        collection=%collection,
        id=%id,
        created=!exists,
        "upsert_request"
    );

    Ok(UpsertResponse {
        id: id.to_string(),
        created: !exists,
        latency_ms: Some(duration.as_millis() as f32),
    })
}
