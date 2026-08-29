//! Shared foundation for every Piramid crate.
//!
//! Errors, configuration, metadata, validation, and telemetry — the things more than one layer
//! needs and none should own. This crate depends on `piramid-compute` only for the
//! [`ExecutionMode`](compute::ExecutionMode) and [`Metric`](compute::Metric) types that
//! configuration carries; it depends on nothing else in the workspace.

pub mod config;
pub mod error;
pub mod metadata;
pub mod telemetry;
pub mod validation;

pub use piramid_compute as compute;

pub use error::{ErrorContext, PiramidError, Result};
