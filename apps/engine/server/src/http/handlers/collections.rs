use axum::{
    extract::{Path, State},
    response::Json,
};

use crate::http::ApiResult as Result;
use crate::services::collection;
use crate::services::types::*;
use crate::state::SharedState;

pub async fn list_collections(
    State(state): State<SharedState>,
) -> Result<Json<CollectionsResponse>> {
    Ok(Json(collection::list_collections(&state)?))
}

pub async fn create_collection(
    State(state): State<SharedState>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<Json<CollectionInfo>> {
    Ok(Json(collection::create_collection(&state, req)?))
}

pub async fn get_collection(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
) -> Result<Json<CollectionInfo>> {
    Ok(Json(collection::get_collection(&state, collection)?))
}

pub async fn delete_collection(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
) -> Result<Json<DeleteCollectionResponse>> {
    Ok(Json(collection::delete_collection(&state, collection)?))
}

pub async fn collection_count(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
) -> Result<Json<CountResponse>> {
    Ok(Json(collection::collection_count(&state, collection)?))
}

pub async fn index_stats(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
) -> Result<Json<IndexStatsResponse>> {
    Ok(Json(collection::index_stats(&state, collection)?))
}

pub async fn rebuild_index(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
) -> Result<Json<RebuildIndexResponse>> {
    Ok(Json(collection::rebuild_index(&state, collection)?))
}

pub async fn find_duplicates(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
    Json(req): Json<DuplicateRequest>,
) -> Result<Json<DuplicateResponse>> {
    Ok(Json(collection::find_duplicates(&state, collection, req)?))
}

pub async fn compact_collection(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
) -> Result<Json<RebuildIndexResponse>> {
    Ok(Json(collection::compact_collection(&state, collection)?))
}

pub async fn rebuild_index_status(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
) -> Result<Json<RebuildIndexStatusResponse>> {
    Ok(Json(collection::rebuild_index_status(&state, collection)?))
}
