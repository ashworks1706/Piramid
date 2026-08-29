use std::time::Instant;

use uuid::Uuid;

use crate::runtime::SharedState;
use crate::services::convert::metadata_to_json;
use crate::services::types::{ListVectorsQuery, VectorResponse};
use crate::services::VECTOR_NOT_FOUND;
use piramid_core::error::{Result, ServerError};
use piramid_core::telemetry::record_lock_read;

use super::ensure_available;

pub fn get_vector(state: &SharedState, collection: String, id: String) -> Result<VectorResponse> {
    ensure_available(state)?;

    let collection_handle = state.get_existing_collection(&collection)?;
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ServerError::InvalidRequest("Invalid UUID".to_string()))?;

    let lock_start = Instant::now();
    let collection_guard = collection_handle.read();
    record_lock_read(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    let entry = collection_guard
        .get(&uuid)?
        .ok_or(ServerError::NotFound(VECTOR_NOT_FOUND.to_string()))?;
    Ok(VectorResponse {
        id: entry.id.to_string(),
        vector: entry.try_get_vector()?,
        text: entry.text,
        metadata: metadata_to_json(&entry.metadata),
    })
}

pub fn list_vectors(
    state: &SharedState,
    collection: String,
    params: ListVectorsQuery,
) -> Result<Vec<VectorResponse>> {
    ensure_available(state)?;

    let collection_handle = state.get_existing_collection(&collection)?;
    let lock_start = Instant::now();
    let collection_guard = collection_handle.read();
    record_lock_read(
        state.collection_manager.tracker(&collection).as_deref(),
        lock_start,
    );

    collection_guard
        .get_all()?
        .into_iter()
        .skip(params.offset)
        .take(params.limit)
        .map(|entry| {
            Ok(VectorResponse {
                id: entry.id.to_string(),
                vector: entry.try_get_vector()?,
                text: entry.text,
                metadata: metadata_to_json(&entry.metadata),
            })
        })
        .collect()
}
