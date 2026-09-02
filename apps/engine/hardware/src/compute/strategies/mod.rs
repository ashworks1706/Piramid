//! Strategy registry: one file per strategy, one arm in [`for_mode`].

mod binary;
mod parallel;
mod scalar;
mod simd;

#[cfg(feature = "gpu-cuda")]
mod cuda;

pub use binary::BinaryStrategy;
pub use parallel::ParallelStrategy;
pub use scalar::ScalarStrategy;
pub use simd::SimdStrategy;

#[cfg(feature = "gpu-cuda")]
pub use cuda::CudaStrategy;

use crate::compute::error::ComputeResult;
use crate::compute::kernels::DistanceKernels;
use crate::compute::mode::ExecutionMode;

static SCALAR: ScalarStrategy = ScalarStrategy;
static SIMD: SimdStrategy = SimdStrategy;
static PARALLEL: ParallelStrategy = ParallelStrategy;
static BINARY: BinaryStrategy = BinaryStrategy;

#[cfg(feature = "gpu-cuda")]
static CUDA: CudaStrategy = CudaStrategy;

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
                return Err(crate::compute::error::ComputeError::StrategyUnavailable {
                    strategy: "gpu",
                    reason: "built without the `gpu-cuda` feature".to_string(),
                });
            }
        }
    };

    if strategy.is_available() {
        Ok(strategy)
    } else {
        Err(crate::compute::error::ComputeError::StrategyUnavailable {
            strategy: strategy.name(),
            reason: "not available on this machine".to_string(),
        })
    }
}
