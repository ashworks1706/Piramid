//! Device memory: [`DeviceBuffer`] lets vectors and weights be uploaded once and reused.

use std::marker::PhantomData;

use crate::gpu::device::Device;
use crate::gpu::error::GpuResult;
use crate::gpu::stream::Stream;

/// A typed allocation in device memory, generic over element type (`f32`, `f16`, `u32`, ...).
#[derive(Debug)]
pub struct DeviceBuffer<T> {
    device: Device,
    handle: DeviceAllocation,
    len: usize,
    _marker: PhantomData<T>,
}

/// Backend-owned pointer to a device allocation, kept opaque to callers.
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
        let handle = crate::gpu::backends::allocate(device, size_bytes)?;
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
        crate::gpu::backends::copy_to_device(&self.device, &mut self.handle, as_bytes(src), stream)
    }

    /// Copy this buffer's contents into `dst`. Enqueued on `stream`.
    pub fn copy_to_host(&self, dst: &mut [T], stream: &Stream) -> GpuResult<()> {
        crate::gpu::backends::copy_to_host(&self.device, &self.handle, as_bytes_mut(dst), stream)
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
        let _ = crate::gpu::backends::free(&self.device, &mut self.handle);
    }
}

/// Reinterpret a typed slice as bytes for transfer.
#[allow(unsafe_code)]
fn as_bytes<T: Copy>(src: &[T]) -> &[u8] {
    // SAFETY: `T: Copy` has no drop glue or interior invariants that a byte view can violate, and
    // the resulting slice borrows `src` for its whole lifetime with a length derived from it.
    unsafe { std::slice::from_raw_parts(src.as_ptr().cast::<u8>(), std::mem::size_of_val(src)) }
}

/// Reinterpret a typed slice as mutable bytes for transfer.
#[allow(unsafe_code)]
fn as_bytes_mut<T: Copy>(dst: &mut [T]) -> &mut [u8] {
    let size = std::mem::size_of_val(dst);
    // SAFETY: as in `as_bytes`; the exclusive borrow of `dst` is preserved by the returned slice.
    unsafe { std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<u8>(), size) }
}
