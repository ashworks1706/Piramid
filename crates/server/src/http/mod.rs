//! HTTP transport.
//!
//! Routes, handlers, request ids, error mapping, and the Prometheus scrape endpoint. Handlers stay
//! thin: parse the request, call a service, serialize the result.
//!
//! DTOs live in `services::types`. The old `http::types` shim, which only re-exported them, is
//! gone so there is one canonical path per type.

pub mod error;
pub mod handlers;
pub mod helpers;
pub mod prometheus;
pub mod request_id;
pub mod routes;

pub use error::{ApiError, ApiResult};
pub use helpers::{json_to_metadata, metadata_to_json};
pub use routes::create_router;
