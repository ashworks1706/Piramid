# Roadmap

## Now (v0.3.0) — make the batch path reachable

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
- [ ] a device-side top-k, so only `k` ordinals and scores cross PCIe. Without it the kernel
      computes N scores and copies all N back, and the transfer eats the win — a faster kernel
      that moves more data is not a faster query
- [ ] keep a device-resident candidate slab across queries, measured against per-call upload.
      This is the experiment, not an optimisation on top of one: the CPU bench is already
      memory-bound at 8192 candidates, so the GPU's edge is HBM bandwidth (~1–3 TB/s against
      ~50–100 GB/s), which exists only if the slab is already there. Pay PCIe per query and the
      whole advantage is spent before the kernel runs
- [ ] write down the VRAM budget the co-located design has to fit in. A 7B model at fp16 is
      ~14 GB and an 8k KV cache ~2 GB, so a 24 GB card leaves ~6–8 GB — about 2–2.5M vectors at
      768 dims and fp32. That ceiling is the scope: single-collection, single-tenant workloads,
      not web-scale search. Decide it deliberately rather than discovering it

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
- [ ] implement `QuantizationLevel::Int4` and `Float16`, or drop the variants. `validate` rejects
      both today

## Next (v0.4.0) — the integrated baseline

The question the whole project turns on: at equal answer quality and context budget, when does
device-resident retrieval inside the inference loop beat a separate vector store and model server?
One model, one GPU, batch size one, no HTTP. Answer it before building anything to serve it.

- [ ] `candle` behind `inference-candle`, weights on the same device retrieval uses
- [ ] an interoperability test proving `piramid-gpu` and `candle` can share one device, context and
      allocator. Assumed today; if it is false, most of this section changes shape
- [ ] a forward-pass driver with `RetrievalHook` call sites from the first commit
- [ ] the first real `RetrievalHook` implementation, as its own crate
- [ ] an end-to-end benchmark — embed, search, top-k, fetch, tokenize, prefill, decode — reporting
      TTFT, tokens/sec, p50/p95 and recall. Kernel microbenchmarks cannot answer the question above
- [ ] measure four configurations against it: split process; in-process CPU index; in-process
      device-resident index; and retrieval overlapped with prefill on its own stream
- [ ] publish the result either way. Co-location losing to a split process, because the index
      competes with weights and KV for bandwidth, is a real finding and the likelier one
- [ ] spans on kernel launches and forward-pass stages

## Next (v0.4.0) — define "fused" before measuring it

"Fused vs prompt-stuffed" is four experiments, in ascending order of difficulty. Do them in order
and stop when one wins; each is a separate measurement against the baseline above.

- [ ] retrieved text appended to the prompt — the control arm, not a result
- [ ] retrieved documents pre-tokenized, so retrieval skips tokenization on the hot path
- [ ] precomputed document KV states reused at prefill. The hard part is that document KV is not
      context-independent: position and preceding context change it, so states cannot simply be
      concatenated. Read CacheBlend on selective recomputation before starting
- [ ] hidden-state fusion through `RetrievalHook` — the only arm that needs the seam at all

## Later (v0.5.0) — serving

Nothing here is needed to answer the v0.4 question. It is what turns the answer into a runtime.

- [ ] paged KV cache
- [ ] `/api/infer` and an OpenAI-compatible `/v1/chat/completions`
- [ ] SSE streaming
- [ ] continuous batching
- [ ] an in-process embedding provider, `provider: piramid`, once `candle` is loaded —
      encoder-only, no KV cache or sampling, reusing the device retrieval already holds. A fourth
      option beside `openai` (including any server speaking that format) and `ollama`, not a
      replacement: a self-hosted embedder stays a supported setup. It needs its own crate, since
      `embeddings` must not depend on `inference`; the binary wires it in through
      `EmbeddingsManager`, the same shape as a `RetrievalHook` implementation

## Later (v0.6.0+) — retrieval-native experiments

One at a time, each against the v0.4 baseline.

- [ ] fused kernels combining retrieval encoding and attention in one launch
- [ ] an index co-designed for the attention access pattern
- [ ] fp16/bf16 for weights and stored vectors, no upcasting on the hot path
- [ ] retrieval over the model's own KV history rather than external documents — a different
      problem from document RAG (long-context attention), kept as its own branch. See
      RetrievalAttention
- [ ] retrieval at block boundaries under block-diffusion decoding, where a block is the natural
      retrieval unit instead of a token. Only interpretable once the autoregressive baseline
      exists, or the win cannot be attributed

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
- distributed placement, and `cluster` beyond the local no-op router, until the single-device
  numbers exist. Multi-device is not interesting while one device is unmeasured
- vendor telemetry integrations — [ADR 0011](decisions/0011-open-standards-only.md)
