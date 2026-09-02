use axum::{extract::State, http::StatusCode, response::Json};

use crate::http::ApiResult as Result;
use crate::services::admin;
use crate::services::types::{HealthResponse, MetricsResponse};
use crate::state::SharedState;

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

/// Prometheus scrape endpoint, served at `/metrics` outside the `/api` prefix.
pub async fn prometheus_metrics(
    State(state): State<SharedState>,
) -> Result<([(axum::http::header::HeaderName, &'static str); 1], String)> {
    let snapshot = admin::metrics(&state)?;
    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            piramid_core::observability::prometheus::CONTENT_TYPE,
        )],
        crate::http::prometheus::render(&snapshot),
    ))
}
