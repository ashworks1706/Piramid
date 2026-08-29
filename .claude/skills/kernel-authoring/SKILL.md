---
name: kernel-authoring
description: Add a compute backend or GPU kernel to Piramid. Use when implementing a DistanceKernels backend, writing CUDA kernels, touching apps/engine/hardware/compute or apps/engine/hardware/gpu, or working on vector memory layout and device transfer.
---

# kernel-authoring

How to add compute backends and kernels without breaking the boundaries that make them swappable.

Read `docs/decisions/0003-backend-first-compute-dispatch.md`,
`0004-gpu-owns-device-compute-owns-math.md`, and `0005-contiguous-vector-layout.md` first. They
explain why the shapes below are what they are.

## The boundary

`apps/engine/hardware/gpu` owns the **device**: contexts, buffers, streams, module loading. No math semantics.
`apps/engine/hardware/compute` owns the **math**: what cosine means, which backend runs it.

Both are leaf crates — they depend on nothing else in the workspace. Do not add a dependency to
either; `scripts/check-deps.sh` will reject it, and the reason is that `apps/engine/inference/runtime` needs a
device too and must not reach through retrieval math to get one.

Vendor SDK types (`cudarc`) appear only in `apps/engine/hardware/gpu/src/backends/`. If `cudarc::` shows up
anywhere else, the abstraction has leaked.

## Adding a CPU backend

One file in `apps/engine/hardware/compute/src/backends/`, one arm in `for_mode` in `backends/mod.rs`. Nothing
else in the workspace changes. If you find yourself editing a third file, stop and reconsider.

1. Implement `DistanceKernels`: `mode`, `name`, `is_available`, and the four pairwise methods.
2. Leave the batch methods alone unless you can beat the default. The defaults loop over pairwise,
   which is correct for every CPU backend.
3. Add a variant to `ExecutionMode` and a string in `from_name`/`as_str`.
4. Add a parity test against `ScalarBackend` — same inputs, results within `1e-5`.
5. Add a criterion bench against `ScalarBackend`. A backend with no measurement is a guess.

`is_available` must be honest. `resolve_available` uses it to fall back, and a backend that lies
produces wrong answers instead of a warning.

## Adding a GPU kernel

**The batch signature is the contract:**

```rust
fn cosine_batch(&self, query: &[f32], candidates: &[f32], dim: usize, out: &mut [f32])
    -> ComputeResult<()>;
```

`candidates` is a contiguous row-major slab. Do not change this to `&[Vec<f32>]` — that is what it
used to be, and it is unusable: each row is a separate allocation, so every call would gather into
a staging buffer, and the gather costs more than the kernel saves. `out` is caller-owned so the
buffer survives across queries and can later be pinned.

Order of work:

1. **Device runtime first, kernels second.** `Device`, `DeviceBuffer`, `Stream` must round-trip —
   allocate, upload, download, compare — before any kernel is written. A kernel debugged on top of
   broken transfer is unfixable.
2. Kernel source in `apps/engine/hardware/gpu/src/kernels/<family>.cu`, typed launch wrapper beside it in
   `<family>.rs`. The wrapper owns launch geometry and argument binding, not device lifetime.
3. Wire it in `apps/engine/hardware/compute/src/backends/cuda.rs` by overriding the batch methods only. Leave
   the pairwise methods delegating to CPU — a single-pair distance will never justify a launch.
4. Flip `CudaBackend::is_available` to a real device probe.

**Keep data resident.** The point of `DeviceBuffer` is that a candidate slab uploads once and is
reused. If your benchmark uploads per query, you are measuring PCIe, and CPU will win.

## Benchmarking

Always against the scalar reference, and always report both:

- **Correctness** — max absolute deviation from `ScalarBackend`.
- **Throughput** — with the transfer cost included if the data is not already resident. A number
  that excludes the upload is not the number the query path sees.

State the vector count, dimension, and whether data was device-resident. A speedup without those
is unfalsifiable.

## Traps

- Measuring a GPU path against a *scattered* CPU baseline. Fix the layout first (`VectorSlab`), or
  you will attribute the layout win to the device.
- `#[allow(unsafe_code)]` outside `apps/engine/hardware/gpu` — the security workflow fails on a new site.
- Panicking in a kernel. Dispatch must degrade with a warning, never abort a query.
- Adding a match arm instead of a file. If new hardware means editing existing dispatch code, the
  registry pattern has been broken.
