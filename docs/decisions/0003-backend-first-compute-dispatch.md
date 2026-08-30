# 0003 — Backend-first compute, trait dispatch

**Context.** `compute/` was organized operation-first: `{cosine,dot,euclidean}/{scalar,simd,
parallel,binary,jit}.rs`, fifteen files. Adding a backend meant editing three directories and four
`match` arms. Dispatch was a `match` on an `ExecutionMode` enum that lived in `config/`, so the
kernel layer imported application configuration. The `Gpu` arm was `panic!("not implemented")` in
all three modules — reachable from a YAML file, so `execution_mode: gpu` crashed the server on the
first search.

`jit.rs` was not JIT. It was `match len { 128 => …, 1536 => … }` over hand-unrolled functions,
which is what the SIMD path and LLVM already do.

**Decision.** Invert to backend-first. One file per backend under `compute/backends/`, each
implementing the whole `DistanceKernels` trait; one arm per backend in the registry. Adding a
backend touches one new file and one existing one.

`ExecutionMode` moves to `compute` and is re-exported by `config` — it names a backend, which is a
compute concern. `compute` now depends on nothing in the workspace.

Dispatch returns `Result` rather than panicking. `jit` is removed.

**Amended.** `resolve_available`, which fell back to the best CPU backend with a `warn`, is gone:
a silent substitution meant a machine without AVX2 produced scalar numbers under a `simd` label.
`for_mode` is the only lookup and it errors. The `#[serde(alias = "Jit")]` compatibility shim is
also gone; there were no sidecars written before the rename.

**Consequences.** Four reachable panics gone. New hardware is a new file, never a new match arm.
The batch methods define the device contract — see [0005](0005-contiguous-vector-layout.md).

**Not decided.** Whether `binary` survives. It is a lossy pre-filter with nothing wired to it yet;
it stays until quantization reaches the search path or it is deleted.
