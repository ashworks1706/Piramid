//! GET /api/version.

use axum::response::Json;

use crate::services::api::VersionResponse;

/// Binary version, plus the git hash of the build when one was baked in.
pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION"),
        git_commit: option_env!("GIT_COMMIT_HASH"),
    })
}
