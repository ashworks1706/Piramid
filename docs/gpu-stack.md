# GPU Stack Scaffold

This document defines the scaffold and ownership boundaries for NVIDIA GPU work in Piramid.

It is intentionally implementation-light: it explains where code should go before kernels and runtime integrations are fully built.

## Goals

- Keep Piramid Rust-native.
- Keep transport/service/cluster code independent of kernel details.
- Allow custom-kernel experimentation without destabilizing the main runtime.

## Ownership Boundaries

- `gpu/` owns GPU runtime glue, backend adapters, kernel launch contracts, and GPU error types.
- `compute/` owns math semantics and CPU/GPU dispatch decisions.
- `inference/` owns transformer execution concerns (batching, streaming, KV-cache ownership).
- `cluster/` owns routing/placement, not kernel or model runtime internals.

## Stack Direction

- Primary GPU toolchain: `cudarc`.
- Transformer inference runtime: `candle`.

This means:

- `cudarc` is the default path for NVIDIA host-side GPU integration and kernel launch plumbing.
- `candle` remains scoped to `inference/` and should not leak into retrieval/index code paths.

## Planned Scaffold Layout

No implementation requirement yet; this is only a target structure:

```text
src/
  gpu/
    mod.rs
    backends/
      mod.rs
      cudarc.rs            # default kernel/runtime adapter
    kernels/
      mod.rs
      distance/            # cosine/dot/euclidean kernels
      indexing/            # IVF/HNSW helpers
      quantization/        # optional quantized ops
  inference/
    mod.rs
    backends/
      mod.rs
      candle.rs
```

## Feature-Flag Scaffold (Planned)

Use additive feature flags so experiments stay isolated:

- `gpu-cudarc`
- `inference-candle`

Keep default builds CPU-safe unless GPU flags are enabled.

## First Milestones (Scaffold Only)

1. Backend trait boundaries are stable in `gpu/` (already started).
2. Add compile-time feature gates for backend selection.
3. Add smoke tests that validate backend selection/fallback behavior without requiring real kernels.
4. Add benchmark harness stubs that can compare CPU path vs GPU path once kernels land.
