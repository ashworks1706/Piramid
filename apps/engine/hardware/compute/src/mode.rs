//! Execution-mode selection: [`ExecutionMode`] names which strategy runs a kernel.

use serde::{Deserialize, Serialize};

/// Which execution strategy should run a kernel; `Auto` is a request, not a strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// Detect the best available strategy at runtime.
    #[default]
    Auto,
    /// Portable scalar reference implementation.
    Scalar,
    /// Explicitly vectorized CPU path (AVX2 / NEON via `wide`).
    Simd,
    /// Rayon-parallel CPU path, for vectors large enough to amortize the fan-out.
    Parallel,
    /// 1-bit quantized approximation. Lossy; intended for cheap pre-filtering.
    Binary,
    /// GPU device execution.
    Gpu,
}

impl ExecutionMode {
    /// Resolve `Auto` into a concrete strategy using detected CPU features; other modes pass through.
    pub fn resolve(&self) -> ExecutionMode {
        match self {
            ExecutionMode::Auto => {
                #[cfg(target_arch = "x86_64")]
                {
                    if is_x86_feature_detected!("avx2") {
                        ExecutionMode::Simd
                    } else {
                        ExecutionMode::Scalar
                    }
                }

                #[cfg(target_arch = "aarch64")]
                {
                    if std::arch::is_aarch64_feature_detected!("neon") {
                        ExecutionMode::Simd
                    } else {
                        ExecutionMode::Scalar
                    }
                }

                #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
                {
                    ExecutionMode::Scalar
                }
            }
            ExecutionMode::Simd => {
                #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
                {
                    ExecutionMode::Simd
                }
                #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
                {
                    ExecutionMode::Scalar
                }
            }
            other => *other,
        }
    }

    /// Stable lowercase name, used by config parsing and telemetry labels.
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionMode::Auto => "auto",
            ExecutionMode::Scalar => "scalar",
            ExecutionMode::Simd => "simd",
            ExecutionMode::Parallel => "parallel",
            ExecutionMode::Binary => "binary",
            ExecutionMode::Gpu => "gpu",
        }
    }

    /// Parse a mode from a config string. Unknown values yield `None`.
    pub fn from_name(name: &str) -> Option<ExecutionMode> {
        match name {
            "auto" => Some(ExecutionMode::Auto),
            "scalar" => Some(ExecutionMode::Scalar),
            "simd" => Some(ExecutionMode::Simd),
            "parallel" => Some(ExecutionMode::Parallel),
            "binary" => Some(ExecutionMode::Binary),
            "gpu" => Some(ExecutionMode::Gpu),
            _ => None,
        }
    }
}
