//! Shared foundation for every Piramid crate: errors, configuration, validation, stats.

pub mod clock;
pub mod config;
pub mod error;
pub mod stats;
pub mod validation;

pub use piramid_hardware::compute;

pub use error::{ErrorContext, PiramidError, Result};
