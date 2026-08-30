//! Strategy registry: one file per strategy, one arm in [`for_mode`].

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

/// Every strategy compiled into this build, available or not, for admin/introspection surfaces.
pub fn all() -> Vec<&'static dyn DistanceKernels> {
    // `mut` is only used by the feature-gated push below.
    #[allow(unused_mut)]
    let mut strategies: Vec<&'static dyn DistanceKernels> =
        vec![&SCALAR, &SIMD, &PARALLEL, &BINARY];
    #[cfg(feature = "gpu-cuda")]
    strategies.push(&CUDA);
    strategies
}

/// The strategy serving `mode` (resolving `Auto` first); errors rather than silently falling back.
pub fn for_mode(mode: ExecutionMode) -> ComputeResult<&'static dyn DistanceKernels> {
    let strategy: &'static dyn DistanceKernels = match mode.resolve() {
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
                return Err(crate::error::ComputeError::StrategyUnavailable {
                    strategy: "gpu",
                    reason: "built without the `gpu-cuda` feature".to_string(),
                });
            }
        }
    };

    if strategy.is_available() {
        Ok(strategy)
    } else {
        Err(crate::error::ComputeError::StrategyUnavailable {
            strategy: strategy.name(),
            reason: "not available on this machine".to_string(),
        })
    }
}
