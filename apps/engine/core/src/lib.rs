//! Shared foundation for every Piramid crate: errors, configuration, metadata, validation,
//! the counters the engine keeps about itself, and where those counters go.

pub mod clock;
pub mod config;
pub mod document;
pub mod error;
pub mod metadata;
pub mod observability;
pub mod stats;
pub mod validation;

pub use document::{Document, Hit};
pub use error::{ErrorContext, PiramidError, Result};
