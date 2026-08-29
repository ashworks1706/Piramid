//! Device runtime errors.

use std::fmt::{Display, Formatter};

/// A device operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuError {
    /// No usable device, or the backend was not compiled in.
    Unavailable(String),
    /// Device memory allocation failed.
    Allocation(String),
    /// A host/device transfer failed or was mis-shaped.
    Transfer(String),
    /// Loading a compiled module or resolving a kernel symbol failed.
    ModuleLoad(String),
    /// A kernel launch failed.
    Launch(String),
    /// The driver or runtime reported an error.
    Runtime(String),
}

impl Display for GpuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(m) => write!(f, "GPU unavailable: {m}"),
            Self::Allocation(m) => write!(f, "GPU allocation failed: {m}"),
            Self::Transfer(m) => write!(f, "GPU transfer failed: {m}"),
            Self::ModuleLoad(m) => write!(f, "GPU module load failed: {m}"),
            Self::Launch(m) => write!(f, "GPU launch failed: {m}"),
            Self::Runtime(m) => write!(f, "GPU runtime error: {m}"),
        }
    }
}

impl std::error::Error for GpuError {}

/// Convenience alias for device results.
pub type GpuResult<T> = std::result::Result<T, GpuError>;
