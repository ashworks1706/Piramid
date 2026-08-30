//! Errors raised by compute kernels.

use std::fmt::{Display, Formatter};

/// A kernel failed to run: dimension mismatch, unavailable strategy, or a device fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputeError {
    /// The requested strategy is not compiled in, or the hardware it needs is absent.
    StrategyUnavailable {
        /// Strategy that was requested.
        strategy: &'static str,
        /// Why it could not be used.
        reason: String,
    },
    /// Operand shapes disagree.
    ShapeMismatch {
        /// What the kernel expected.
        expected: usize,
        /// What it received.
        got: usize,
    },
    /// A quantized vector's encoding is internally inconsistent and cannot be decoded, or a
    /// quantization level has no encoder.
    InvalidEncoding(String),
    /// The underlying vendor backend ran but failed.
    Backend {
        /// Vendor backend that failed.
        backend: &'static str,
        /// Underlying message.
        message: String,
    },
}

impl Display for ComputeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StrategyUnavailable { strategy, reason } => {
                write!(f, "compute strategy '{strategy}' unavailable: {reason}")
            }
            Self::ShapeMismatch { expected, got } => {
                write!(f, "compute shape mismatch: expected {expected}, got {got}")
            }
            Self::InvalidEncoding(message) => {
                write!(f, "invalid quantized encoding: {message}")
            }
            Self::Backend { backend, message } => {
                write!(f, "compute backend '{backend}' failed: {message}")
            }
        }
    }
}

impl std::error::Error for ComputeError {}

/// Convenience alias for kernel results.
pub type ComputeResult<T> = std::result::Result<T, ComputeError>;
