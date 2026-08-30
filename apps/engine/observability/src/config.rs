//! Observability errors.
//!
//! The settings themselves live in `core::config::TelemetryConfig`, in the startup block of the
//! configuration file, because they are installed once and there is one place to set them.

use thiserror::Error;

/// Telemetry could not be initialised.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct ObservabilityError(pub String);

/// Convenience alias for observability results.
pub type ObservabilityResult<T> = Result<T, ObservabilityError>;
