//! HTTP error mapping.
//!
//! `piramid-core::PiramidError` is transport-agnostic — no library crate knows what a status code
//! is. [`ApiError`] is a transport-layer newtype that maps an [`ErrorKind`] onto a status and
//! renders a JSON body. It converts from `PiramidError`, so a handler returns [`ApiResult<T>`],
//! keeps `?`, and everything below it still returns `piramid-core::Result`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use piramid_core::error::{ErrorKind, PiramidError};
use serde_json::json;

/// A [`PiramidError`] rendered as an HTTP response.
#[derive(Debug)]
pub struct ApiError(pub PiramidError);

/// Handler result type.
pub type ApiResult<T> = std::result::Result<T, ApiError>;

impl<E: Into<PiramidError>> From<E> for ApiError {
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

/// Map a transport-agnostic error kind onto an HTTP status.
fn status_for(kind: ErrorKind) -> StatusCode {
    match kind {
        ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
        ErrorKind::NotFound => StatusCode::NOT_FOUND,
        ErrorKind::Conflict => StatusCode::CONFLICT,
        ErrorKind::Unauthenticated => StatusCode::UNAUTHORIZED,
        ErrorKind::Forbidden => StatusCode::FORBIDDEN,
        ErrorKind::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        ErrorKind::Timeout => StatusCode::REQUEST_TIMEOUT,
        ErrorKind::Upstream => StatusCode::BAD_GATEWAY,
        ErrorKind::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = status_for(self.0.kind());
        let body = Json(json!({
            "error": self.0.to_string(),
            "code": status.as_u16(),
        }));
        (status, body).into_response()
    }
}
