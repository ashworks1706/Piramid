# Roadmap

## Now (v0.3.0) — make the batch path reachable

- [x] add a criterion bench for `compute` — scalar vs SIMD vs parallel, realistic dimensions
- [ ] split `ExecutionMode`'s two axes. The bench settles it: SIMD is 4.5–5.8x scalar pairwise
      across 384–3072 dims, and `Parallel` is *slower than scalar at every dimension and every
      candidate count measured* — 0.29x at 384, 0.08x at 1536, 0.43–0.46x throughout the batch
      group. It chunks within a single vector at `max(len/ncpus, 1024)`, so a realistic embedding
      is one or two chunks of trivial work behind a rayon fan-out. "SIMD across all cores" —
      parallel over rows, vectorized within a row — is the configuration that cannot be asked for
      and the only one worth having (ADR 0013)
- [ ] decide whether `Parallel` survives that split at all. As an execution mode over a single
      vector, the bench found no regime where it wins
- [ ] route the flat index scan through `cosine_batch` instead of a per-vector loop
- [ ] route the rerank loop in `search::engine` through the batch API

## Now (v0.3.0) — contiguous layout

- [ ] make `collections::cache::VectorStore` contiguous: one `Vec<f32>` at a fixed stride plus a
      `Uuid → u32` ordinal map, implementing `VectorReader::as_slab`. Removal is the open design
      question — a slab cannot cheaply delete a row, so it needs a tombstone or swap-remove story
      settled alongside HNSW's. Write it with tests this time
- [ ] re-run the compute bench: scattered vs contiguous candidates, same kernel. The batch group
      already shows the ceiling to beat — SIMD's advantage falls from 5.1x at 128 candidates to
      2.7x at 8192, where the slab is 25 MB and the kernel is memory-bound rather than
      compute-bound
- [ ] use `u32` ordinals instead of `Uuid` in HNSW adjacency lists (sidecar format bump + load path)

## Now (v0.3.0) — GPU

- [ ] wire `cudarc` behind `gpu-cuda`: a real `Device`, `DeviceBuffer`, `Stream`
- [ ] round-trip test — allocate, upload, download, compare — before any kernel
- [ ] a `cosine_batch` CUDA kernel, benched against the scalar reference
- [ ] keep a device-resident candidate slab across queries, measure against per-call upload

## Now (v0.3.0) — independent

- [ ] wire `piramid-compute::quantization` into the search path, starting with a PQ distance
      kernel in `compute/strategies/` beside `binary`
- [ ] binary pre-filter into full-precision rerank, with a recall measurement — and in the same
      change move `BinaryStrategy` out of `DistanceKernels`. It returns a different, approximate
      answer rather than the same one faster, so it is a recall trade, not an execution strategy
      (ADR 0013)
- [ ] backfill doc comments so `missing_docs` can move from `allow` to `warn` (~860 items)
- [ ] wire graceful shutdown. `AppState::initiate_shutdown` and `checkpoint_all` exist and are
      complete; six call sites already reject requests once the flag is set. Nothing calls either
      — `axum::serve` runs without `with_graceful_shutdown` and there is no signal handler, so
      every SIGTERM is a hard kill. The WAL means no data loss, but every restart replays a log a
      clean checkpoint would have truncated, and in-flight writes die mid-request
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
- [ ] an in-process embedding provider, `provider: piramid`, once `candle` is loaded —
      encoder-only, no KV cache or sampling, reusing the device retrieval already holds. A fourth
      option beside `openai` (including any server speaking that format) and `ollama`, not a
      replacement: a self-hosted embedder stays a supported setup. It needs its own crate, since
      `embeddings` must not depend on `inference`; the binary wires it in through
      `EmbeddingsManager`, the same shape as a `RetrievalHook` implementation
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

- [ ] migrate off `serde_yaml` 0.9 — archived upstream and unmaintained. It parses every config
      file and the support bundle. `serde_norway` is the maintained fork
- [ ] `piramid write-config` emits `Config::default()` rather than the commented
      `config.example.yaml`, so a generated file has the values without the explanations
- [ ] make `runtime:` reach further. A reload applies to collections opened after it, so a running
      collection keeps the search, cache and WAL settings it opened with. Either re-read them per
      request or say in the docs that it is per-collection-open
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
- [ ] no test starts the server and reloads config. The startup-block guard and the env-override
      precedence are covered at the loader level only

## Out of scope

- a managed cloud service
- non-NVIDIA GPU backends until the CUDA path is real
- a second deployable process — [ADR 0001](decisions/0001-single-binary.md)
- vendor telemetry integrations — [ADR 0011](decisions/0011-open-standards-only.md)
