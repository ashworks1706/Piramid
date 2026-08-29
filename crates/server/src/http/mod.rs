//! HTTP transport.
//!
//! Routes, handlers, request ids, and the error mapping. Handlers stay thin: parse the request,
//! call a service, serialize the result.

pub mod error;
pub mod handlers; // endpoint logic
pub mod helpers; // utility functions and macros
pub mod request_id;
pub mod routes; // wires handlers to URL paths

// DTOs live in `services::types`; the old `http::types` re-export shim is gone so there is one
// canonical path per type.

pub use error::{ApiError, ApiResult};
pub use helpers::{json_to_metadata, metadata_to_json};
pub use routes::create_router;
