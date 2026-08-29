//! GPU boundary and backend selection surface.
//!
//! This module stays implementation-light for now. It defines backend contracts and separates
//! backend adapters from kernel namespaces so CUDA integration can grow without leaking into
//! services, cluster routing, or collection domain logic.

pub mod backends;
pub mod kernels;
mod types;

pub use backends::{CudarcBackend, DefaultGpuBackend};
pub use types::{GpuBackend, GpuError};
