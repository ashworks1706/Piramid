//! Shared foundation for every Piramid crate.
//!
//! Errors, configuration, metadata, validation, and self-measurement. Depends on
//! `piramid-compute` for the [`ExecutionMode`](compute::ExecutionMode) and
//! [`Metric`](compute::Metric) types configuration carries, and on nothing else in the workspace.

pub mod clock;
pub mod config;
pub mod error;
pub mod metadata;
pub mod stats;
pub mod validation;

pub use piramid_compute as compute;

pub use error::{ErrorContext, PiramidError, Result};
