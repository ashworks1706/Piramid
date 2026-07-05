//! Backend adapters for GPU execution.
//!
//! `cuda_oxide` is the default scaffold path. `cudarc` remains available as an optional
//! interoperability adapter.

mod cuda_oxide;
mod cudarc;

pub use cuda_oxide::CudaOxideBackend;
pub use cudarc::CudarcInteropBackend;

pub type DefaultGpuBackend = CudaOxideBackend;
