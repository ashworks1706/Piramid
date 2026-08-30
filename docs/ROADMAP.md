# Roadmap

## Now (v0.3.0) — make the batch path reachable

- [ ] add unit tests for `VectorSlab` — push, replace, ordinals, row bounds, gather
- [ ] add a criterion bench for `compute` — scalar vs SIMD vs parallel, realistic dimensions
- [ ] route the flat index scan through `cosine_batch` instead of a per-vector loop
- [ ] route the rerank loop in `search::engine` through the batch API

## Now (v0.3.0) — contiguous layout

- [ ] migrate `CacheManager` onto `VectorSlab`, make `SlabVectorReader` the default reader
- [ ] re-run the compute bench: scattered vs contiguous candidates, same kernel
- [ ] use `u32` ordinals instead of `Uuid` in HNSW adjacency lists (sidecar format bump + load path)

## Now (v0.3.0) — GPU

- [ ] wire `cudarc` behind `gpu-cuda`: a real `Device`, `DeviceBuffer`, `Stream`
- [ ] round-trip test — allocate, upload, download, compare — before any kernel
- [ ] a `cosine_batch` CUDA kernel, benched against the scalar reference
- [ ] keep a device-resident candidate slab across queries, measure against per-call upload

## Now (v0.3.0) — independent

- [ ] wire `piramid-storage::quantization` into the search path
- [ ] binary pre-filter into full-precision rerank, with a recall measurement
- [ ] backfill doc comments so `missing_docs` can move from `allow` to `warn` (~860 items)
- [ ] decide what happens to `cluster`
- [ ] implement `QuantizationLevel::Int4` and `Float16`, or drop the variants. `validate` rejects
      both today

## Next (v0.4.0)

- [ ] retrofit a small model in Python, measure fused vs prompt-stuffed at equal token budget
- [ ] publish the result either way
- [ ] `candle` behind `inference-candle`, weights on the same device retrieval uses
- [ ] a forward-pass driver with `RetrievalHook` call sites from the first commit
- [ ] paged KV cache
- [ ] the first real `RetrievalHook` implementation, as its own crate
- [ ] `/api/infer` and an OpenAI-compatible `/v1/chat/completions`
- [ ] SSE streaming
- [ ] continuous batching
- [ ] spans on kernel launches and forward-pass stages

## Later (v0.5.0+)

- [ ] fused kernels combining retrieval encoding and attention in one launch
- [ ] an index co-designed for the attention access pattern
- [ ] fp16/bf16 for weights and stored vectors, no upcasting on the hot path
- [ ] distributed placement in `cluster`

## Unscheduled

- [ ] tune IVF, or say in the docs that HNSW is the one to use
- [ ] migrate bincode 1.x → 2.x: a format migration for records, sidecars and the WAL, with a read
      path for existing data (RUSTSEC-2025-0141, ignored in `deny.toml` until then)
- [ ] cut the website copy down to what the product actually does today (the seven
      components are all reachable; it is the claims that are ahead of the code)
- [ ] make the npm and python SDKs real clients, or unpublish them. They also predate the
      one-shape request bodies, so whatever ships has to speak the current API
- [ ] decide on authentication and rate limiting. `ErrorKind::Unauthenticated` maps to 401 and
      nothing emits it; CORS allows any origin, method and header; `/config/reload` is open
- [ ] test `apps/cli`. It has none, and `support-bundle` is the path that must never emit a secret
- [ ] one HTTP-level test that builds the router and hits it. 25 routes, no test constructs one
- [ ] untrack `apps/sdk/python/dist/` and `piramid.egg-info/` — build artifacts committed to git

## Out of scope

- a managed cloud service
- pretraining models from scratch
- competing with Qdrant on vector-database breadth
- non-NVIDIA GPU backends until the CUDA path is real
- a second deployable process — [ADR 0001](decisions/0001-single-binary.md)
- vendor telemetry integrations — [ADR 0011](decisions/0011-open-standards-only.md)
