//! Execution-mode selection for compute kernels.
//!
//! [`ExecutionMode`] names *which backend* should run a kernel. It lives in `compute/` rather than
//! `config/` so the kernel layer stays a leaf: `config/` re-exports this type for callers, but
//! `compute/` never depends on application configuration.

use serde::{Deserialize, Serialize};

/// Which compute backend should execute a kernel.
///
/// [`ExecutionMode::Auto`] is a request, not a backend: call [`ExecutionMode::resolve`] to turn it
/// into a concrete choice based on detected CPU features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecutionMode {
    /// Detect the best available backend at runtime.
    #[default]
    Auto,
    /// Portable scalar reference implementation.
    Scalar,
    /// Explicitly vectorized CPU path (AVX2 / NEON via `wide`).
    ///
    /// The `Jit` alias maps here: `Jit` was a dimension-specialized unrolling of this same code
    /// path and was removed. The alias keeps previously persisted index sidecars loadable.
    #[serde(alias = "Jit")]
    Simd,
    /// Rayon-parallel CPU path, for vectors large enough to amortize the fan-out.
    Parallel,
    /// 1-bit quantized approximation. Lossy; intended for cheap pre-filtering.
    Binary,
    /// GPU device execution.
    Gpu,
}

impl ExecutionMode {
    /// Resolve [`ExecutionMode::Auto`] into a concrete backend using detected CPU features.
    ///
    /// This performs no availability checking for non-`Auto` modes: an explicitly requested
    /// backend is returned as-is so the caller can distinguish "unavailable" from "not asked for".
    /// Use [`crate::backends::resolve_available`] to get a mode that is guaranteed to be
    /// runnable on this machine.
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
            // `jit` retained as an accepted spelling; it now selects the SIMD backend.
            "simd" | "jit" => Some(ExecutionMode::Simd),
            "parallel" => Some(ExecutionMode::Parallel),
            "binary" => Some(ExecutionMode::Binary),
            "gpu" => Some(ExecutionMode::Gpu),
            _ => None,
        }
    }

    /// Whether the resolved backend uses explicit vectorization.
    pub fn use_simd(&self) -> bool {
        matches!(
            self.resolve(),
            ExecutionMode::Simd | ExecutionMode::Parallel
        )
    }

    /// Whether the resolved backend fans out across threads.
    pub fn use_parallel(&self) -> bool {
        matches!(self.resolve(), ExecutionMode::Parallel)
    }
}
