use axum::{extract::State, response::Json};

use crate::http::ApiResult as Result;
use crate::runtime::SharedState;
use crate::services::admin;
use crate::services::types::ReadyzResponse;

pub async fn readyz(State(state): State<SharedState>) -> Result<Json<ReadyzResponse>> {
    Ok(Json(admin::readyz(&state)?))
}
