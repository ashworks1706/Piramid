# 0013 — Strategies are not backends

**Context.** `compute/backends/` held five files: `scalar`, `simd`, `parallel`, `binary`, `cuda`.
Three of them are the same hardware doing the same arithmetic at different speeds, so opening that
folder promised a hardware diversity four of its five files do not have.

Worse, the word had drifted. By this point the tree had three `backends/` directories meaning
three different things:

| Folder | A "backend" is |
|---|---|
| `gpu/backends/` | a vendor SDK — `cudarc`, and ROCm or Metal later |
| `inference/backends/` | a model runtime — `candle` |
| `compute/backends/` | any implementation of `DistanceKernels`, mostly on one CPU |

The first two match how the rest of the field uses the word. The third does not, and
[0010](0010-name-audit.md) exists to stop exactly this: one name, one meaning. That audit could
not have caught it — when it ran, `compute` owned the only `backends/` in the tree.

Two further problems surfaced from the same reading. `binary` is not an execution strategy at all:
it computes Hamming agreement over sign bits and never multiplies two floats, so it returns a
*different, approximate* answer rather than the same one faster. And `ParallelBackend` runs a
scalar inner loop inside its rayon chunks, so vectorization and threading — genuinely orthogonal —
are collapsed into one enum where "SIMD across all cores", the fastest CPU configuration, cannot
be expressed.

**Decision.** `compute/backends/` becomes `compute/strategies/`. `ExecutionMode` already means
"execution strategy", so the crate now reads consistently: `kernels.rs` is the contract,
`metric.rs` is what to measure, `strategies/` is how it runs. `ComputeError::BackendUnavailable`
follows as `StrategyUnavailable`; `ComputeError::Backend` keeps its name because it is exactly the
vendor-failure case.

"Backends" now means one thing repo-wide — the swappable vendor layer — and lives only in `gpu`
and `inference`. `strategies/cuda.rs` and `gpu/backends/cudarc.rs` stop sharing a name, which is
right: one is "run this on the GPU", the other is "talk to NVIDIA", and only the second may name
a `cudarc` type.

**Not done, deliberately.** The other two findings are on the roadmap rather than in this change,
because both are behaviour and one needs a measurement:

- Moving `binary` out of `DistanceKernels` into the search pre-filter path. It is a recall/latency
  trade, not a hardware one, and the roadmap already wants it measured against full-precision
  rerank. Until then it stays where it is with a doc comment saying it is the odd one out.
- Splitting `ExecutionMode`'s axes so vectorization and threading compose. This is the one with
  real performance upside and it needs the compute bench — also on the roadmap — to justify a
  shape before the enum changes again.

**Consequences.** A folder rename, one error variant renamed, and doc comments through the crate;
no behaviour change and no dependency-rule change. The tree no longer implies that
`scalar`/`simd`/`parallel` are different backends, and the next contributor looking for the vendor
layer finds it in one place.
