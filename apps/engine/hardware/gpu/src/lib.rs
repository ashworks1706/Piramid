#![deny(missing_docs)]

//! Device runtime: contexts, memory, streams, and compiled kernels, shared by `compute` and
//! `inference` so both can use one [`Device`].

pub mod backends;
pub mod buffer;
pub mod device;
pub mod error;
pub mod kernels;
pub mod module;
pub mod stream;

pub use buffer::{DeviceAllocation, DeviceBuffer};
pub use device::{Device, DeviceCapabilities, DeviceRuntime};
pub use error::{GpuError, GpuResult};
pub use module::{KernelModule, LaunchConfig};
pub use stream::Stream;
