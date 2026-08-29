# Roadmap

What's being built and in what order. Pick one scoped item. If your idea is adjacent but not
listed, open an issue before implementing.

Current state is v0.2.0: a working single-node vector database. Collections, vector CRUD, kNN and
range search, metadata filtering, embedding ingestion, WAL and checkpoints, three ANN index
families, SIMD distance kernels. Eleven crates with an enforced dependency rule. Inference is
scaffolding.

The order below is a dependency order, not a wish list. Each block needs the one above it.

## Now (v0.3.0)

The goal of this release is that a GPU kernel, when written, has somewhere to plug in and a
baseline to be measured against. Neither is true today.

### 1. Make the batch path reachable

Every scoring call site currently uses the pairwise API — `metric.calculate(a, b, mode)` in
`flat/index.rs`, `hnsw/index.rs`, and `search/engine.rs`. Nothing calls
`DistanceKernels::cosine_batch`. A GPU kernel written now would be code nothing invokes, so this
comes first.

- [ ] add unit tests for `VectorSlab` — push, replace, ordinals, row bounds, gather. It's about
      180 lines with no coverage, and everything below builds on it
- [ ] add a criterion bench for `compute`, comparing scalar, SIMD, and parallel across realistic
      dimensions. There is no compute bench today, only `hnsw_performance`, so there is nothing to
      measure a change against
- [ ] route the flat index scan through `cosine_batch` rather than a per-vector loop. Flat is the
      right first consumer because it already touches every vector
- [ ] route the rerank loop in `search::engine` through the batch API

### 2. Contiguous layout

Now measurable, because step 1 gave us a bench and a batch call site.

- [ ] migrate `CacheManager` onto `VectorSlab` and make `SlabVectorReader` the default reader
- [ ] re-run the compute bench: scattered versus contiguous candidates, same kernel. This number
      is the one that justifies GPU work, or doesn't
- [ ] use `u32` ordinals instead of `Uuid` inside HNSW adjacency lists. Changes the sidecar
      format, so it needs a version bump and a load path for old files

### 3. GPU

Only meaningful once 1 and 2 land. Order matters here too: a kernel debugged on top of broken
transfer is unfixable.

- [ ] wire `cudarc` behind `gpu-cuda`: a real `Device`, `DeviceBuffer`, and `Stream`
- [ ] round-trip test — allocate, upload, download, compare — before writing any kernel
- [ ] a `cosine_batch` CUDA kernel, benched against the scalar reference for parity and speed
- [ ] keep a device-resident candidate slab across queries and measure against per-call upload

### Independent of the above

These don't block anything and can be picked up in any order.

- [ ] wire the existing PQ implementation into the search path. `piramid-storage::quantization` is
      fully implemented and has zero callers outside its own module
- [ ] binary pre-filter into full-precision rerank, with a recall measurement
- [ ] fix the 21 `unwrap`/`expect` call sites outside tests, then flip `unwrap_used` and
      `expect_used` to `deny`
- [ ] backfill doc comments so `missing_docs` can move from `allow` to `warn`. Around 860 public
      items, concentrated in `server/services/types` (165) and `core/config` (roughly 130).
      `compute`, `gpu`, and `inference` already pass and enforce it themselves
- [ ] decide what happens to `cluster`. It always returns `RouteDecision::Local` and is threaded
      through `AppState` in six places for no behaviour

### Done

- [x] Prometheus text format at `/metrics`
- [x] spans on search, write, embed, rebuild, and compact
- [x] `piramid support-bundle` for bug reports
- [x] `#![deny(missing_docs)]` on `compute`, `gpu`, and `inference`

## Next (v0.4.0)

Decide the mechanism before building it:

- [ ] retrofit a small model in Python — JetBrains-Research/project-RETRO fixes the train/test
      leakage in the original — and measure fused against prompt-stuffed at equal token budget, on
      a corpus with low train/test overlap
- [ ] publish the result either way, since a negative one redirects the project cheaply

If it holds:

- [ ] `candle` behind `inference-candle`, loading weights onto the same device retrieval uses
- [ ] a forward-pass driver with `RetrievalHook` call sites from the first commit
- [ ] paged KV cache
- [ ] the first real `RetrievalHook` implementation, as its own crate depending on both
      `piramid-inference` and `piramid-search`
- [ ] `/api/infer` and an OpenAI-compatible `/v1/chat/completions`
- [ ] SSE streaming
- [ ] continuous batching
- [ ] spans on kernel launches and forward-pass stages, which is when OTLP starts earning its place

## Later (v0.5.0 and beyond)

- [ ] fused kernels combining retrieval encoding and attention in one launch
- [ ] an index co-designed for the attention access pattern, with relevance scoring jointly learned
- [ ] fp16/bf16 for weights and stored vectors, with no upcasting on the hot path
- [ ] distributed placement in `cluster`, once inference exists and dictates where things live

## Out of scope

Don't build toward these without a decision recorded in `docs/decisions/`.

A managed cloud service. Piramid is a binary you run.

Pretraining models from scratch. Retrofit only; pretraining isn't a solo project.

Competing with Qdrant on vector-database breadth. Multi-tenancy, sharding, replication, and hybrid
keyword search aren't differentiators here. If pure vector search is what someone needs, they
should use Qdrant.

Non-NVIDIA GPU backends until the CUDA path is real. `gpu/backends/` is structured so ROCm or
Metal is additive, but a second backend before the first one works is scaffolding.

A second deployable process. See [ADR 0001](decisions/0001-single-binary.md).

Vendor telemetry integrations. Protocols are in scope, products aren't. See
[ADR 0011](decisions/0011-open-standards-only.md).

## Known gaps

Things that are wrong or missing today, verified against the code rather than remembered.

- The batch kernel API has no callers. `DistanceKernels::cosine_batch` and its siblings exist and
  every scoring path uses the pairwise methods instead.
- `VectorSlab` has no tests, and `as_slab`/`gather_into` have no callers. The seam is defined and
  unused.
- There is no benchmark for `compute`. The only bench in the workspace is `hnsw_performance`.
- `quantization` is fully implemented and unreachable from the search path.
- `cluster` always routes locally and is carried through `AppState` for nothing.
- IVF works but is untuned. HNSW covers the sizes we actually test at.
- 21 `unwrap`/`expect` call sites outside tests, so `unwrap_used` is still `allow`.
- Roughly 860 undocumented public items outside `compute`, `gpu`, and `inference`.
- The website is fourteen components for a product whose headline feature isn't built.
- The npm and python SDKs are 11 and 7 lines, published under names already claimed on their
  registries. Either make them real clients or unpublish them; a stub under an installable name is
  worse than no package.
