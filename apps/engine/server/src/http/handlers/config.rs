use axum::{extract::State, response::Json};

use crate::http::ApiResult as Result;
use crate::runtime::SharedState;
use crate::services::admin;
use crate::services::types::{ConfigReloadResponse, ConfigStatusResponse};

pub async fn config_status(State(state): State<SharedState>) -> Result<Json<ConfigStatusResponse>> {
    Ok(Json(admin::config_status(&state)?))
}

pub async fn reload_config(State(state): State<SharedState>) -> Result<Json<ConfigReloadResponse>> {
    Ok(Json(admin::reload_config(&state)?))
}
