use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};

use crate::http::request_id::RequestId;
use crate::http::ApiResult as Result;
use crate::services::types::*;
use crate::services::vector;
use crate::state::SharedState;

pub async fn insert_vector(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
    Json(req): Json<InsertRequest>,
) -> Result<Json<InsertResponse>> {
    Ok(Json(vector::insert_vector(&state, collection, req)?))
}

pub async fn get_vector(
    State(state): State<SharedState>,
    Path((collection, id)): Path<(String, String)>,
) -> Result<Json<VectorResponse>> {
    Ok(Json(vector::get_vector(&state, collection, id)?))
}

pub async fn list_vectors(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
    Query(params): Query<ListVectorsQuery>,
) -> Result<Json<Vec<VectorResponse>>> {
    Ok(Json(vector::list_vectors(&state, collection, params)?))
}

pub async fn delete_vector(
    State(state): State<SharedState>,
    Path((collection, id)): Path<(String, String)>,
) -> Result<Json<DeleteResponse>> {
    Ok(Json(vector::delete_vector(&state, collection, id)?))
}

pub async fn delete_vectors(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
    Json(req): Json<DeleteVectorsRequest>,
) -> Result<Json<DeleteResponse>> {
    Ok(Json(vector::delete_vectors(&state, collection, req)?))
}

pub async fn search_vectors(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
    Extension(request_id): Extension<RequestId>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>> {
    Ok(Json(vector::search_vectors(
        &state,
        collection,
        request_id.0.as_str(),
        req,
    )?))
}

pub async fn upsert_vector(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
    Json(req): Json<UpsertRequest>,
) -> Result<Json<UpsertResponse>> {
    Ok(Json(vector::upsert_vector(&state, collection, req)?))
}

pub async fn range_search_vectors(
    State(state): State<SharedState>,
    Path(collection): Path<String>,
    Extension(request_id): Extension<RequestId>,
    Json(req): Json<RangeSearchRequest>,
) -> Result<Json<SearchResponse>> {
    Ok(Json(vector::range_search_vectors(
        &state,
        collection,
        request_id.0.as_str(),
        req,
    )?))
}
