//! Device runtime: contexts, memory, streams, and compiled kernels.
//!
//! # What belongs here
//!
//! Everything about *talking to a device* and nothing about what the math means. Vendor SDK types
//! are confined to [`backends`]; every layer above sees only [`Device`], [`DeviceBuffer`],
//! [`Stream`], and [`KernelModule`].
//!
//! # Why it is not part of `compute/`
//!
//! Two subsystems need a device: [`piramid_compute`] for distance kernels and [`piramid_inference`]
//! for model execution. If the device runtime lived inside `compute/`, inference would have to
//! depend on the retrieval math layer to allocate memory. Keeping `gpu/` a peer means both can
//! share one [`Device`] — which is the whole point, since vectors and model weights need to sit in
//! the same address space for retrieval and generation to meet without a host round-trip.
//!
//! ```text
//! compute/backends/cuda.rs ──┐
//!                            ├──> gpu/  (Device, DeviceBuffer, Stream, KernelModule)
//! inference/backends/*.rs  ──┘
//! ```

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
