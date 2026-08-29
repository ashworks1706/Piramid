# 0004 — `gpu` owns the device, `compute` owns the math

**Context.** The previous `gpu/` module defined a `GpuBackend` trait with
`cosine_similarity_batch(&self, query: &[f32], candidates: &[Vec<f32>])`. Two problems: the
signature takes scattered allocations, which no device can use without a per-call gather; and it
put distance semantics in the GPU module while CPU distance semantics lived in `compute`. The same
operation had two homes, chosen by hardware.

There was also no device abstraction at all — no `Device`, no `DeviceBuffer`, no `Stream`. Every
call would have been host → device → host, which forecloses inference, where weights must *stay*
resident.

**Decision.** Split by concern, not by hardware.

`gpu` owns the device runtime only: `Device`, `DeviceBuffer<T>`, `Stream`, `KernelModule`, and
kernel sources with typed launch wrappers. No math semantics. Vendor SDK types are confined to
`gpu/backends/`.

`compute` owns math and backend selection. Its CUDA backend is a thin adapter that calls into
`gpu`.

Both are leaf crates, which is the point: `inference` needs a device too, and must not depend on
retrieval math to allocate memory.

```
compute/backends/cuda.rs ──┐
                           ├──→ gpu
inference/backends/*.rs  ──┘
```

**Consequences.** Retrieval and generation can share one `Device`, so vectors and model weights
land in one address space — the thing the single-process design exists for. A second vendor
backend (ROCm, Metal) is a new module under `gpu/backends/` and touches nothing above.

`crates/gpu` is the one crate where `unsafe_code` is allowed; the workspace denies it everywhere
else, with two audited exceptions.

**Not decided.** Whether kernels are written in `.cu` compiled to PTX, or emitted from Rust. The
layout assumes `.cu` beside a `.rs` wrapper, which is reversible.
