# Roadmap

What we're building and in what order. Pick one scoped item. If your idea is adjacent but not
listed, open an issue before implementing.

Status: **v0.2.0** — a working single-node vector database. Collections, vector CRUD, kNN and range
search, metadata filtering, embedding ingestion, WAL and checkpoints, three ANN index families,
SIMD distance kernels. Eleven crates with enforced layering. Inference is scaffolding.

---

## Now — v0.3.0: make the seams real

Infrastructure that pays off regardless of which fusion mechanism wins
([0006](decisions/0006-retrieval-fusion-seam.md)).

**Vector layout**
- [ ] migrate `CacheManager` onto `VectorSlab`; make `SlabVectorReader` the default reader
- [ ] benchmark SIMD before/after — scattered vs contiguous candidates, same kernel
- [ ] use `u32` ordinals instead of `Uuid` inside HNSW adjacency lists (sidecar format change)

**GPU device runtime**
- [ ] wire `cudarc` behind `gpu-cuda`: real `Device`, `DeviceBuffer`, `Stream`
- [ ] round-trip test: allocate, upload, download, compare — no kernels yet
- [ ] `cosine_batch` CUDA kernel, benched against the scalar reference for parity and speed
- [ ] keep a device-resident candidate slab across queries; measure against per-call upload

**Quantization**
- [ ] wire the existing PQ implementation into the search path (currently orphaned)
- [ ] binary pre-filter → full-precision rerank, with a recall measurement

**Observability**
- [ ] Prometheus text format at `/metrics`
- [ ] spans on search, write, embed, and kernel launches

---

## Next — v0.4.0: the fusion experiment

**Decide the mechanism before building it.**
- [ ] retrofit a small model in Python (JetBrains-Research/project-RETRO, which fixes the
      train/test leakage in the original) and measure fused vs prompt-stuffed at equal token
      budget, on a corpus with low train/test overlap
- [ ] publish the result either way — a negative result redirects the project cheaply

**If it holds:**
- [ ] `candle` behind `inference-candle`: load weights onto the same `Device` retrieval uses
- [ ] forward-pass driver with `RetrievalHook` call sites from the first commit
- [ ] paged KV cache
- [ ] first real `RetrievalHook` implementation
- [ ] `/api/infer` and an OpenAI-compatible `/v1/chat/completions`
- [ ] SSE streaming
- [ ] continuous batching

---

## Later — v0.5.0+

- [ ] fused kernels: retrieval encoding + attention in one launch
- [ ] index co-designed for the attention access pattern — relevance scoring jointly learned
- [ ] fp16/bf16 for weights and stored vectors, no upcasting on the hot path
- [ ] distributed placement in `cluster` — after inference exists and dictates where things live

---

## Out of scope

Don't build toward these without an explicit decision recorded in `docs/decisions/`.

- **A managed cloud service.** Piramid is a binary you run.
- **Pretraining models from scratch.** Retrofit only; pretraining is not a solo project.
- **Competing with Qdrant on vector-database breadth.** Multi-tenancy, sharding, replication, and
  hybrid keyword search are not differentiators for us. If pure vector search is what someone
  needs, they should use Qdrant.
- **Non-NVIDIA GPU backends** until the CUDA path is real. `gpu/backends/` is structured so ROCm or
  Metal is additive, but a second backend before the first one works is scaffolding.
- **A second deployable process.** See [0001](decisions/0001-single-binary.md).

---

## Known gaps

Honest list of things that are wrong or missing today.

- `cluster` always returns `RouteDecision::Local` and is threaded into `AppState` for nothing.
- `quantization` is implemented but reachable from nothing in the search path.
- `IVF` works but is untuned; HNSW covers the sizes we actually test at.
- `missing_docs` is `allow` at the workspace level — the newer crates enforce it, the older ones
  have not been backfilled.
- `unwrap_used` and `expect_used` are `allow`; ~20 call sites outside tests need fixing before
  they can be flipped to `deny`.
- README documents `/api/infer`, which does not exist.
- The website is 14 components for a product whose headline feature is unbuilt.
