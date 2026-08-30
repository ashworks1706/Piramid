//! Backend registry.
//!
//! One file per backend, one arm per backend in [`for_mode`]. Adding a backend touches this file
//! and nothing else.

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

/// Every backend compiled into this build, available or not, for admin/introspection surfaces.
pub fn all() -> Vec<&'static dyn DistanceKernels> {
    // `mut` is only used by the feature-gated push below.
    #[allow(unused_mut)]
    let mut backends: Vec<&'static dyn DistanceKernels> = vec![&SCALAR, &SIMD, &PARALLEL, &BINARY];
    #[cfg(feature = "gpu-cuda")]
    backends.push(&CUDA);
    backends
}

/// The backend serving `mode`, resolving `Auto` first.
///
/// Errors if the mode names a backend this build does not contain or this machine cannot run.
/// There is no fallback: a caller that asked for a specific backend and silently got a different
/// one has no way to know its numbers came from somewhere else.
pub fn for_mode(mode: ExecutionMode) -> ComputeResult<&'static dyn DistanceKernels> {
    let backend: &'static dyn DistanceKernels = match mode.resolve() {
        ExecutionMode::Scalar | ExecutionMode::Auto => &SCALAR,
        ExecutionMode::Simd => &SIMD,
        ExecutionMode::Parallel => &PARALLEL,
        ExecutionMode::Binary => &BINARY,
        ExecutionMode::Gpu => {
            #[cfg(feature = "gpu-cuda")]
            {
                &CUDA
            }
            #[cfg(not(feature = "gpu-cuda"))]
            {
                return Err(crate::error::ComputeError::BackendUnavailable {
                    backend: "gpu",
                    reason: "built without the `gpu-cuda` feature".to_string(),
                });
            }
        }
    };

    if backend.is_available() {
        Ok(backend)
    } else {
        Err(crate::error::ComputeError::BackendUnavailable {
            backend: backend.name(),
            reason: "not available on this machine".to_string(),
        })
    }
}
