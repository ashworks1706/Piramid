//! HTTP transport, use-case orchestration, and shared runtime state.
//!
//! - [`http`] — axum routes, handlers, request ids. Handlers stay thin: parse, call a service,
//!   serialize.
//! - [`services`] — use cases and the canonical API DTOs. Services know about runtime state and
//!   domain objects, never about file formats or index internals.
//! - [`runtime`] — [`AppState`], the process-wide shared state.
//! - [`cluster`] — node identity and routing. Local-only today.

pub mod cluster;
pub mod http;
pub mod runtime;
pub mod services;

pub use http::create_router;
pub use runtime::{AppState, SharedState};
