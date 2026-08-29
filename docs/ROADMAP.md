# Roadmap

What's being built and in what order. Pick one scoped item. If your idea is adjacent but not
listed, open an issue before implementing.

Current state is v0.2.0: a working single-node vector database. Collections, vector CRUD, kNN and
range search, metadata filtering, embedding ingestion, WAL and checkpoints, three ANN index
families, SIMD distance kernels. Eleven crates with an enforced dependency rule. Inference is
scaffolding.

## Now (v0.3.0)

Infrastructure that pays off whichever fusion mechanism turns out to be right. See
[ADR 0006](decisions/0006-retrieval-fusion-seam.md).

Vector layout:

- [ ] migrate `CacheManager` onto `VectorSlab` and make `SlabVectorReader` the default reader
- [ ] benchmark SIMD before and after, scattered versus contiguous candidates with the same kernel
- [ ] use `u32` ordinals instead of `Uuid` inside HNSW adjacency lists (changes the sidecar format)

GPU device runtime:

- [ ] wire `cudarc` behind `gpu-cuda`: a real `Device`, `DeviceBuffer`, and `Stream`
- [ ] round-trip test — allocate, upload, download, compare — before writing any kernel
- [ ] a `cosine_batch` CUDA kernel, benched against the scalar reference for parity and speed
- [ ] keep a device-resident candidate slab across queries and measure it against per-call upload

Quantization:

- [ ] wire the existing PQ implementation into the search path, where it currently isn't used
- [ ] binary pre-filter into full-precision rerank, with a recall measurement

Observability:

- [x] Prometheus text format at `/metrics`
- [x] spans on search, write, embed, rebuild, and compact
- [ ] spans on kernel launches, once there are kernels to launch

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
- [ ] the first real `RetrievalHook` implementation
- [ ] `/api/infer` and an OpenAI-compatible `/v1/chat/completions`
- [ ] SSE streaming
- [ ] continuous batching

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

Things that are wrong or missing today.

- `cluster` always returns `RouteDecision::Local` and is threaded into `AppState` for nothing.
- `quantization` is implemented but nothing in the search path reaches it.
- IVF works but is untuned. HNSW covers the sizes we actually test at.
- `missing_docs` is `allow` at the workspace level. The newer crates enforce it; the older ones
  haven't been backfilled.
- `unwrap_used` and `expect_used` are `allow`. Around twenty call sites outside tests need fixing
  before they can be flipped to `deny`.
- The website is fourteen components for a product whose headline feature isn't built.
- The npm and python SDKs are 11 and 7 lines, published under names already claimed on their
  registries. Either make them real clients or unpublish them; a stub under an installable name is
  worse than no package.
