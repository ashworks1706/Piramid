//! HTTP transport: routes, handlers, request ids, error mapping, the scrape endpoint.
//!
//! Handlers stay thin — parse the request, call a service, serialize the result. DTOs live in
//! `services::api`.

pub mod error;
pub mod handlers;
pub mod prometheus;
pub mod request_id;
pub mod routes;

pub use error::{ApiError, ApiResult};
pub use routes::create_router;
