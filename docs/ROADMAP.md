# Roadmap

Piramid is an inference engine for RAG: one process holding the documents, the model weights and the
KV cache on one device, so retrieval can run *during* generation rather than once before it.

That last part is the whole point. Retrieving once before prefill saves a service hop worth
milliseconds against seconds of generation — real, but small. Retrieving repeatedly inside a single
generation, overlapped with compute, against device-resident state, cannot be done across a service
boundary at all. Everything below is ordered to make that measurable, then shippable.

The index lives in-process because it has to, not because Piramid is a database. Grow the parts the
device path uses; hold the rest at maintenance.

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

The slab comes before any CUDA. A kernel written against today's `HashMap`-backed store benchmarks
the gather, not the kernel.

- [ ] make `collections::cache::VectorStore` contiguous: one `Vec<f32>` at a fixed stride plus a
      `Uuid → u32` ordinal map, implementing `VectorReader::as_slab`. Stable ordinals with a
      tombstone bitset rather than swap-remove — a moved row means repairing every adjacency list
      that referenced it, and stable ordinals are what every later device representation assumes.
      Write it with tests this time
- [ ] re-run the compute bench: scattered vs contiguous candidates, same kernel. The batch group
      already shows the ceiling to beat — SIMD's advantage falls from 5.1x at 128 candidates to
      2.7x at 8192, where the slab is 25 MB and the kernel is memory-bound rather than
      compute-bound

## Now (v0.3.0) — the device-resident query path

The gate for everything above it. No fusion, no differentiator, no interesting serving story until
a search runs end to end on the GPU without touching host memory.

- [ ] measure bandwidth contention first. Run the CPU search kernel concurrently with a synthetic
      memory-bound workload and watch both degrade; repeat on the device once the kernel exists.
      Decode is bandwidth-bound — it reads every weight per token — so a search kernel streaming
      the slab competes for exactly the resource generation is starved on. If co-located retrieval
      costs more than it saves, that is the central finding, and it is a week of work to learn
      rather than a year
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
- [ ] PQ on the device, with measured recall. VRAM is the binding constraint on the entire design:
      a 7B model at fp16 is ~14 GB and an 8k KV cache ~2 GB, so a 24 GB card leaves ~6–8 GB —
      about 2–2.5M vectors at 768 dims and fp32. PQ at 8x takes that to ~16–20M, which is the
      difference between demoing on a toy corpus and demoing on a real one. Not a side quest
- [ ] IVF is the GPU family; HNSW stays the CPU path. Graph traversal is pointer-chasing and
      parallelises badly on a device, which is why FAISS-GPU ships IVF-Flat and IVF-PQ and leaves
      HNSW on the host. Tune IVF against a fixed dataset with recall reported, and say so in the
      docs rather than leaving the choice open
- [ ] binary pre-filter into full-precision rerank, with a recall measurement — and in the same
      change move `BinaryStrategy` out of `DistanceKernels`. It returns a different, approximate
      answer rather than the same one faster, so it is a recall trade, not an execution strategy
      (ADR 0013)

## Now (v0.3.0) — what `serve` has to have before it is exposed

`piramid serve` is the deliverable, which makes these correctness and security work rather than
polish. A runtime other people run cannot ship as it stands.

- [ ] wire graceful shutdown. `AppState::initiate_shutdown` and `checkpoint_all` exist and are
      complete; six call sites already reject requests once the flag is set. Nothing calls either
      — `axum::serve` runs without `with_graceful_shutdown` and there is no signal handler, so
      every SIGTERM is a hard kill. The WAL means no data loss, but every restart replays a log a
      clean checkpoint would have truncated, and in-flight writes die mid-request
- [ ] authentication and rate limiting. `ErrorKind::Unauthenticated` maps to 401 and nothing emits
      it; CORS allows any origin, method and header; `/config/reload` is open to anyone who can
      reach the port
- [ ] one HTTP-level test that builds the router and hits it. 25 routes, no test constructs one
- [ ] test `apps/cli`. It has none, and `support-bundle` is the path that must never emit a secret
- [ ] migrate bincode 1.x → 2.x: a format migration for records, sidecars and the WAL, with a read
      path for existing data (RUSTSEC-2025-0141, ignored in `deny.toml` until then)
- [ ] migrate off `serde_yaml` 0.9 — archived upstream and unmaintained. It parses every config
      file and the support bundle. `serde_norway` is the maintained fork
- [ ] implement `QuantizationLevel::Int4` and `Float16`, or drop the variants. `validate` rejects
      both today

## Next (v0.4.0) — the integrated baseline

One model, one GPU, batch size one, no HTTP. Weeks rather than months: `candle-transformers`
already has the model, the KV cache and sampling. The cost here is integration friction, not
implementation volume, which is why the spike comes first.

- [ ] the interop spike, before anything else. Allocate with `piramid-gpu`, hand the pointer to
      candle as a tensor (or the reverse), run a kernel, verify numerics, and confirm in Nsight
      that no host round trip happened. One day. If it fails, everything below changes shape, and
      that is much cheaper to learn before the runner exists than after
- [ ] pin `cudarc` to whatever candle uses, and record that constraint where it will be seen. Two
      crate versions means two unrelated `CudaDevice` types and possibly two contexts, and every
      "shared device" operation silently routes through host memory instead. This puts `gpu`'s
      dependency version under candle's control, which is in tension with `gpu` being a leaf
      (ADR 0004)
- [ ] `candle` behind `inference-candle`, weights on the same device retrieval uses. Qwen is the
      supported model — see v0.6 for why that choice is load-bearing
- [ ] a forward-pass driver with `RetrievalHook` call sites from the first commit (ADR 0015)
- [ ] the first real `RetrievalHook` implementation, as its own crate
- [ ] an end-to-end benchmark — embed, search, top-k, fetch, tokenize, prefill, decode — reporting
      TTFT, tokens/sec, p50/p95 and recall. Kernel microbenchmarks cannot answer the question
- [ ] measure four configurations against it: split process; in-process CPU index; in-process
      device-resident index; retrieval overlapped with prefill on its own stream
- [ ] settle retrieval frequency with numbers. Per-block — every ~32 tokens — is roughly 16 calls
      in a 512-token generation, where brute force over a resident slab is fine. Per-layer is
      32x that and needs a real index at sub-100µs. Which regime v0.6 targets decides how much ANN
      machinery the device path actually owed
- [ ] publish the result either way. Retrieval before prefill is the control arm and is expected
      to show close to nothing — the win, if there is one, is in v0.6. Co-location losing outright
      because the index competes with weights and KV for bandwidth is also a real finding
- [ ] spans on kernel launches and forward-pass stages

## Next (v0.5.0) — `piramid serve`

The runtime becomes something someone else can run: a model, a collection, one command. Co-located
RAG with ordinary models, which is useful without fusion and is simultaneously the baseline v0.6 is
measured against.

- [ ] `piramid serve --model … --collection … --device cuda:0`
- [ ] `/api/infer` and an OpenAI-compatible `/v1/chat/completions`
- [ ] SSE streaming
- [ ] paged KV cache
- [ ] continuous batching
- [ ] an in-process embedding provider, `provider: piramid`, reusing the device retrieval already
      holds — encoder-only, no KV cache or sampling. A fourth option beside `openai` (including any
      server speaking that format) and `ollama`, not a replacement: a self-hosted embedder stays a
      supported setup. It needs its own crate, since `embeddings` must not depend on `inference`;
      the binary wires it in through `EmbeddingsManager`, the same shape as a `RetrievalHook`
      implementation
- [ ] cut the website copy down to what the runtime does by then

## Later (v0.6.0) — retrieval during generation

The differentiator, and the reason for everything above it. No off-the-shelf model can use it, so
Qwen is forked and hooked; other architectures fall back to prompt-stuffing until adapters exist.

- [ ] fork the Qwen model implementation and add `RetrievalPoint` call sites between decoder
      layers. `forward` runs every layer internally, so per-layer intervention means owning one
      model file per architecture and drifting from upstream. Accepted cost, and the reason only
      one model is supported at first
- [ ] retrieval on its own CUDA stream, joined against the model's — the `launch`/`join` split
      exists for exactly this (ADR 0015). Candle does not expose per-op stream control, so this is
      built around it rather than with it
- [ ] retrieval at block boundaries, measured against the v0.5 prompt-stuffed baseline at equal
      token budget. This is the headline number the project exists to produce
- [ ] retrieved documents pre-tokenized, so retrieval skips tokenization on the hot path
- [ ] precomputed document KV states reused at prefill. Document KV is not context-independent:
      position and preceding context change it, so states cannot simply be concatenated. Read
      CacheBlend on selective recomputation before starting
- [ ] a scheduler dividing VRAM and bandwidth between index, weights and KV under load

## Later (v0.7.0+) — beyond one model

- [ ] a trained adapter so fusion works on models Piramid has not forked
- [ ] fused kernels combining retrieval encoding and attention in one launch, once profiling shows
      launch or synchronisation overhead actually matters
- [ ] an index co-designed for the attention access pattern
- [ ] fp16/bf16 for weights and stored vectors, no upcasting on the hot path
- [ ] retrieval over the model's own KV history rather than external documents — a different
      problem from document RAG (long-context attention), kept as its own branch. See
      RetrievalAttention
- [ ] retrieval at block boundaries under block-diffusion decoding, where a block is the natural
      retrieval unit instead of a token. Only interpretable once the autoregressive baseline
      exists, or the win cannot be attributed

## Unscheduled

- [ ] backfill doc comments so `missing_docs` can move from `allow` to `warn` (~860 items)
- [ ] `piramid write-config` emits `Config::default()` rather than the commented
      `config.example.yaml`, so a generated file has the values without the explanations
- [ ] make `runtime:` reach further. A reload applies to collections opened after it, so a running
      collection keeps the search, cache and WAL settings it opened with. Either re-read them per
      request or say in the docs that it is per-collection-open
- [ ] no test starts the server and reloads config. The startup-block guard and the env-override
      precedence are covered at the loader level only
- [ ] untrack `apps/sdk/python/dist/` and `piramid.egg-info/` — build artifacts committed to git

## Out of scope

- a managed cloud service
- non-NVIDIA GPU backends until the CUDA path is real
- a second deployable process — [ADR 0001](decisions/0001-single-binary.md)
- distributed placement, and `cluster` beyond the local no-op router, until the single-device
  numbers exist. Multi-device is not interesting while one device is unmeasured
- vendor telemetry integrations — [ADR 0011](decisions/0011-open-standards-only.md)
- the npm and python SDKs. They predate the one-shape request bodies and nothing depends on them;
  unpublish rather than maintain clients for a runtime whose API is still moving
- growing the vector database past what the device path uses. Storage, the WAL and collections are
  the substrate, not the product, and stay at maintenance
