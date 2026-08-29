//! Process-wide shared state.

pub mod disk;
pub mod state;

pub use state::{AppState, RebuildJobStatus, RebuildState, SharedState};
