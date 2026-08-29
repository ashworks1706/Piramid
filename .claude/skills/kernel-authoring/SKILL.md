---
name: kernel-authoring
description: Add a compute backend or GPU kernel to Piramid. Use when implementing a DistanceKernels backend, writing CUDA kernels, touching hardware/compute or hardware/gpu, or working on vector memory layout and device transfer.
---

# kernel-authoring

How to add compute backends and kernels without breaking the boundaries that make them swappable.

Read `docs/decisions/0003-backend-first-compute-dispatch.md`,
`0004-gpu-owns-device-compute-owns-math.md`, and `0005-contiguous-vector-layout.md` first. They
explain why the shapes below are what they are.

## The boundary

`hardware/gpu` owns the device: contexts, buffers, streams, module loading. No math semantics.
`hardware/compute` owns the math: what cosine means, and which backend runs it.

Both are leaf crates, depending on nothing else in the workspace. Don't add a dependency to either;
`scripts/check-deps.sh` will reject it, and the reason is that `inference` needs a device too and
must not reach through retrieval math to get one.

Vendor SDK types like `cudarc` appear only in `hardware/gpu/src/backends/`. If `cudarc::` shows up
anywhere else, the abstraction has leaked.

## Adding a CPU backend

One file in `hardware/compute/src/backends/`, one arm in `for_mode` in `backends/mod.rs`. Nothing
else in the workspace changes. If you find yourself editing a third file, stop and reconsider.

1. Implement `DistanceKernels`: `mode`, `name`, `is_available`, and the four pairwise methods.
2. Leave the batch methods alone unless you can beat the default. The defaults loop over pairwise,
   which is correct for every CPU backend.
3. Add a variant to `ExecutionMode` and a string in `from_name` and `as_str`.
4. Add a parity test against `ScalarBackend` — same inputs, results within `1e-5`.
5. Add a criterion bench against `ScalarBackend`. A backend with no measurement is a guess.

`is_available` has to be honest. `resolve_available` uses it to fall back, and a backend that lies
produces wrong answers instead of a warning.

## Adding a GPU kernel

The batch signature is the contract:

```rust
fn cosine_batch(&self, query: &[f32], candidates: &[f32], dim: usize, out: &mut [f32])
    -> ComputeResult<()>;
```

`candidates` is a contiguous row-major slab. Don't change this to `&[Vec<f32>]` — that's what it
used to be and it's unusable, because each row is a separate allocation, so every call would gather
into a staging buffer and the gather costs more than the kernel saves. `out` is caller-owned so the
buffer survives across queries and can be pinned later.

Order of work:

1. Device runtime first, kernels second. `Device`, `DeviceBuffer`, and `Stream` have to round-trip
   — allocate, upload, download, compare — before any kernel is written. A kernel debugged on top
   of broken transfer is unfixable.
2. Kernel source in `hardware/gpu/src/kernels/<family>.cu`, with a typed launch wrapper beside it
   in `<family>.rs`. The wrapper owns launch geometry and argument binding, not device lifetime.
3. Wire it in `hardware/compute/src/backends/cuda.rs` by overriding the batch methods only. Leave
   the pairwise methods delegating to CPU; a single-pair distance will never justify a launch.
4. Flip `CudaBackend::is_available` to a real device probe.

Keep data resident. The point of `DeviceBuffer` is that a candidate slab uploads once and gets
reused. If your benchmark uploads per query, you're measuring PCIe and CPU will win.

## Benchmarking

Always against the scalar reference, and always report both:

- Correctness: max absolute deviation from `ScalarBackend`.
- Throughput: with the transfer cost included, if the data isn't already resident. A number that
  excludes the upload isn't the number the query path sees.

State the vector count, dimension, and whether data was device-resident. A speedup without those
can't be checked.

## Traps

- Measuring a GPU path against a scattered CPU baseline. Fix the layout first with `VectorSlab`, or
  you'll credit the layout win to the device.
- `#[allow(unsafe_code)]` outside `hardware/gpu`. The security workflow fails on a new site.
- Panicking in a kernel. Dispatch degrades with a warning; it never aborts a query.
- Adding a match arm instead of a file. If new hardware means editing existing dispatch code, the
  registry pattern has been broken.
