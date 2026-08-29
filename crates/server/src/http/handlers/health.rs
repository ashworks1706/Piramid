use axum::{extract::State, http::StatusCode, response::Json};

use crate::http::ApiResult as Result;
use crate::runtime::SharedState;
use crate::services::admin;
use crate::services::types::{HealthResponse, MetricsResponse};

pub async fn health() -> Json<HealthResponse> {
    Json(admin::health())
}

pub async fn health_embeddings(State(state): State<SharedState>) -> StatusCode {
    if admin::embeddings_available(&state) {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

pub async fn metrics(State(state): State<SharedState>) -> Result<Json<MetricsResponse>> {
    Ok(Json(admin::metrics(&state)?))
}
