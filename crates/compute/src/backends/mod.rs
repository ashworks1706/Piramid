//! Backend registry.
//!
//! One file per backend, one arm per backend in [`for_mode`]. Adding a backend touches this file
//! and nothing else in the crate.

mod binary;
mod parallel;
mod scalar;
mod simd;

#[cfg(feature = "gpu-cuda")]
mod cuda;

pub use binary::BinaryBackend;
pub use parallel::ParallelBackend;
pub use scalar::ScalarBackend;
pub use simd::SimdBackend;

#[cfg(feature = "gpu-cuda")]
pub use cuda::CudaBackend;

use crate::error::ComputeResult;
use crate::kernels::DistanceKernels;
use crate::mode::ExecutionMode;

static SCALAR: ScalarBackend = ScalarBackend;
static SIMD: SimdBackend = SimdBackend;
static PARALLEL: ParallelBackend = ParallelBackend;
static BINARY: BinaryBackend = BinaryBackend;

#[cfg(feature = "gpu-cuda")]
static CUDA: CudaBackend = CudaBackend;

/// Every backend compiled into this build, available or not.
///
/// Intended for admin/introspection surfaces that report what the binary can do.
pub fn all() -> Vec<&'static dyn DistanceKernels> {
    // `mut` is only used by the feature-gated push below.
    #[allow(unused_mut)]
    let mut backends: Vec<&'static dyn DistanceKernels> = vec![&SCALAR, &SIMD, &PARALLEL, &BINARY];
    #[cfg(feature = "gpu-cuda")]
    backends.push(&CUDA);
    backends
}

/// Look up the backend serving `mode`, without checking availability.
///
/// `Auto` is resolved first. Returns [`ComputeError::BackendUnavailable`] when the mode names a
/// backend that was not compiled into this build.
pub fn for_mode(mode: ExecutionMode) -> ComputeResult<&'static dyn DistanceKernels> {
    match mode.resolve() {
        ExecutionMode::Scalar => Ok(&SCALAR),
        ExecutionMode::Simd => Ok(&SIMD),
        ExecutionMode::Parallel => Ok(&PARALLEL),
        ExecutionMode::Binary => Ok(&BINARY),
        ExecutionMode::Gpu => {
            #[cfg(feature = "gpu-cuda")]
            {
                Ok(&CUDA)
            }
            #[cfg(not(feature = "gpu-cuda"))]
            {
                Err(crate::error::ComputeError::BackendUnavailable {
                    backend: "gpu",
                    reason: "built without the `gpu-cuda` feature".to_string(),
                })
            }
        }
        // `resolve` maps Auto onto a concrete mode before this match.
        ExecutionMode::Auto => Ok(&SCALAR),
    }
}

/// Look up a backend that is guaranteed to run on this machine.
///
/// Falls back to the best available CPU backend when the requested one is missing or unavailable,
/// logging once per call at `warn`. Use this on paths that must produce a number; use
/// [`for_mode`] where an unavailable backend should surface as an error instead.
pub fn resolve_available(mode: ExecutionMode) -> &'static dyn DistanceKernels {
    match for_mode(mode) {
        Ok(backend) if backend.is_available() => backend,
        Ok(backend) => {
            let fallback = cpu_default();
            tracing::warn!(
                target: "piramid::compute",
                requested = backend.name(),
                fallback = fallback.name(),
                "requested compute backend is unavailable; falling back"
            );
            fallback
        }
        Err(error) => {
            let fallback = cpu_default();
            tracing::warn!(
                target: "piramid::compute",
                %error,
                fallback = fallback.name(),
                "compute backend lookup failed; falling back"
            );
            fallback
        }
    }
}

/// Best CPU backend for this target, used as the universal fallback.
fn cpu_default() -> &'static dyn DistanceKernels {
    if SIMD.is_available() {
        &SIMD
    } else {
        &SCALAR
    }
}
