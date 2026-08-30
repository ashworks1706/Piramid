//! Configuration loading errors.

use thiserror::Error;

/// Configuration could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    /// `CONFIG_FILE` could not be read or parsed.
    #[error("invalid configuration file: {0}")]
    File(String),
    /// An environment variable held a value the parser rejected.
    #[error("invalid environment configuration: {name}: {reason}")]
    Env {
        /// Variable name.
        name: String,
        /// What was wrong with it.
        reason: String,
    },
    /// The merged configuration failed validation.
    #[error("invalid configuration: {0}")]
    Invalid(String),
}
