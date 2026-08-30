//! Shared foundation for every Piramid crate: errors, configuration, metadata, validation, stats.

pub mod clock;
pub mod config;
pub mod error;
pub mod metadata;
pub mod stats;
pub mod validation;

pub use piramid_compute as compute;

pub use error::{ErrorContext, PiramidError, Result};
