//! Backend adapters for GPU execution.
//!
//! `cudarc` is the default scaffold path for NVIDIA CUDA runtime and kernel integration.

mod cudarc;

pub use cudarc::CudarcBackend;

pub type DefaultGpuBackend = CudarcBackend;
