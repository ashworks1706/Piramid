//! HTTP transport, use-case orchestration, and shared runtime state.
//!
//! - [`http`] — axum routes, handlers, request ids. Handlers stay thin: parse, call a service,
//!   serialize.
//! - [`services`] — everything true because a server exists: admission checks, lock acquisition,
//!   metrics, and the canonical API DTOs. Durability belongs to `collections`, not here.
//! - [`state`] — [`AppState`], the process-wide shared state, and [`disk`] the pressure watcher.
//! - [`cluster`] — node identity and routing. Local-only today.

pub mod cluster;
pub mod disk;
pub mod http;
pub mod services;
pub mod state;

pub use http::create_router;
pub use state::{AppState, SharedState};
