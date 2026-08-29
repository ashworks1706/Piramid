//! Device memory.
//!
//! [`DeviceBuffer`] is the type that makes device-resident data possible. It exists so vectors and
//! model weights can be uploaded **once** and reused across many kernel launches, instead of being
//! copied host→device→host on every call. Any API that forces a per-call upload will be slower
//! than the CPU path it replaces.

use std::marker::PhantomData;

use crate::device::Device;
use crate::error::GpuResult;
use crate::stream::Stream;

/// A typed allocation in device memory.
///
/// Generic over the element type so the same abstraction serves `f32` vector slabs, `f16` model
/// weights, and `u32` index structures.
///
/// # Implementing
///
/// The `handle` field is deliberately opaque. A backend stores its device pointer there; nothing
/// above this module inspects it.
#[derive(Debug)]
pub struct DeviceBuffer<T> {
    device: Device,
    handle: DeviceAllocation,
    len: usize,
    _marker: PhantomData<T>,
}

/// Backend-owned pointer to a device allocation.
///
/// Kept opaque so vendor pointer types never leak into signatures above [`crate`].
#[derive(Debug)]
pub struct DeviceAllocation {
    /// Raw device address, interpreted by the owning backend.
    pub ptr: u64,
    /// Allocation size in bytes.
    pub size_bytes: usize,
}

impl<T: Copy> DeviceBuffer<T> {
    /// Allocate `len` uninitialized elements on `device`.
    pub fn alloc(device: &Device, len: usize) -> GpuResult<Self> {
        let size_bytes = len * std::mem::size_of::<T>();
        let handle = crate::backends::allocate(device, size_bytes)?;
        Ok(Self {
            device: device.clone(),
            handle,
            len,
            _marker: PhantomData,
        })
    }

    /// Allocate and fill from a host slice in one step.
    pub fn from_host(device: &Device, src: &[T], stream: &Stream) -> GpuResult<Self> {
        let mut buffer = Self::alloc(device, src.len())?;
        buffer.copy_from_host(src, stream)?;
        Ok(buffer)
    }

    /// Copy `src` into this buffer. Enqueued on `stream`; may return before the copy completes.
    pub fn copy_from_host(&mut self, src: &[T], stream: &Stream) -> GpuResult<()> {
        crate::backends::copy_to_device(&self.device, &mut self.handle, as_bytes(src), stream)
    }

    /// Copy this buffer's contents into `dst`. Enqueued on `stream`.
    pub fn copy_to_host(&self, dst: &mut [T], stream: &Stream) -> GpuResult<()> {
        crate::backends::copy_to_host(&self.device, &self.handle, as_bytes_mut(dst), stream)
    }

    /// Number of elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the allocation is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Device this buffer lives on.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Backend allocation handle, for kernel launch argument binding.
    pub fn handle(&self) -> &DeviceAllocation {
        &self.handle
    }
}

impl<T> Drop for DeviceBuffer<T> {
    fn drop(&mut self) {
        // Frees are best-effort: a failure here means the driver is already in a bad state, and
        // there is no useful recovery from a destructor.
        let _ = crate::backends::free(&self.device, &mut self.handle);
    }
}

/// Reinterpret a typed slice as bytes for transfer.
fn as_bytes<T: Copy>(src: &[T]) -> &[u8] {
    // SAFETY: `T: Copy` has no drop glue or interior invariants that a byte view can violate, and
    // the resulting slice borrows `src` for its whole lifetime with a length derived from it.
    unsafe { std::slice::from_raw_parts(src.as_ptr().cast::<u8>(), std::mem::size_of_val(src)) }
}

/// Reinterpret a typed slice as mutable bytes for transfer.
fn as_bytes_mut<T: Copy>(dst: &mut [T]) -> &mut [u8] {
    let size = std::mem::size_of_val(dst);
    // SAFETY: as in `as_bytes`; the exclusive borrow of `dst` is preserved by the returned slice.
    unsafe { std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<u8>(), size) }
}
