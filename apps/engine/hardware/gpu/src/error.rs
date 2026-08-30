//! Device runtime errors.

use thiserror::Error;

/// A device operation failed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GpuError {
    /// No usable device, or the backend was not compiled in.
    #[error("GPU unavailable: {0}")]
    Unavailable(String),
    /// Device memory allocation failed.
    #[error("GPU allocation failed: {0}")]
    Allocation(String),
    /// A host/device transfer failed or was mis-shaped.
    #[error("GPU transfer failed: {0}")]
    Transfer(String),
    /// Loading a compiled module or resolving a kernel symbol failed.
    #[error("GPU module load failed: {0}")]
    ModuleLoad(String),
    /// A kernel launch failed.
    #[error("GPU launch failed: {0}")]
    Launch(String),
    /// The driver or runtime reported an error.
    #[error("GPU runtime error: {0}")]
    Runtime(String),
}

/// Convenience alias for device results.
pub type GpuResult<T> = std::result::Result<T, GpuError>;
