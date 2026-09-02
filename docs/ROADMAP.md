# Roadmap

Piramid is an inference engine for RAG: one process holding the documents, the model weights and the
KV cache on one device, so retrieval can run during generation rather than once before it.

## Now (v0.3.0) — make the batch path reachable

- [ ] split `ExecutionMode`'s two axes into kernel and row scheduling, so "SIMD across all cores"
      can be asked for. Measured: SIMD is 4.5–5.8x scalar pairwise across 384–3072 dims;
      `Parallel` is 0.29x at 384, 0.08x at 1536, and 0.43–0.46x across the batch group. It chunks
      within a single vector at `max(len/ncpus, 1024)`, so a realistic embedding is one or two
      chunks behind a rayon fan-out
- [ ] decide whether `Parallel` survives the split. The bench found no dimension or candidate count
      where it beats scalar
- [ ] route the flat index scan through `cosine_batch` instead of a per-vector loop
- [ ] route the rerank loop in `retrieval::search::engine` through the batch API

## Now (v0.3.0) — contiguous layout

- [ ] make `collections::cache::VectorStore` contiguous: one `Vec<f32>` at a fixed stride plus a
      `Uuid → u32` ordinal map, implementing `VectorReader::as_slab`. Stable ordinals with a
      tombstone bitset rather than swap-remove; a moved row invalidates every adjacency list
      referencing it
- [ ] re-run the compute bench: scattered vs contiguous candidates, same kernel. Current ceiling —
      SIMD's advantage over scalar falls from 5.1x at 128 candidates to 2.7x at 8192, where the
      slab is 25 MB and the kernel is memory-bound

## Now (v0.3.0) — the device-resident query path

- [ ] measure bandwidth contention: run the search kernel concurrently with a memory-bound
      workload and record the degradation of each. Decode reads every weight per token, so a
      search kernel streaming the slab contends for the same bandwidth
- [ ] wire `cudarc` behind `gpu-cuda`: a real `Device`, `DeviceBuffer`, `Stream`
- [ ] round-trip test — allocate, upload, download, compare — before any kernel
- [ ] a `cosine_batch` CUDA kernel, benched against the scalar reference
- [ ] a device-side top-k, so only `k` ordinals and scores cross PCIe
- [ ] keep a device-resident candidate slab across queries, measured against per-call upload. The
      CPU bench is already memory-bound at 8192 candidates, so the GPU's margin is HBM bandwidth
      (~1–3 TB/s against ~50–100 GB/s), available only while the slab stays resident
- [ ] PQ on the device, with measured recall. A 7B model at fp16 is ~14 GB and an 8k KV cache
      ~2 GB, leaving ~6–8 GB on a 24 GB card: ~2–2.5M vectors at 768 dims and fp32, or ~16–20M at
      8x compression
- [ ] tune IVF against a fixed dataset with recall reported, and document IVF as the device family
      and HNSW as the host family. Graph traversal parallelises poorly on a device; FAISS-GPU ships
      IVF-Flat and IVF-PQ and leaves HNSW on the CPU
- [ ] binary pre-filter into full-precision rerank, with a recall measurement, moving
      `BinaryStrategy` out of `DistanceKernels` in the same change — it returns an approximate
      answer, so it is a recall trade rather than an execution strategy

## Now (v0.3.0) — what `serve` needs before it is exposed

- [ ] wire graceful shutdown. `AppState::initiate_shutdown` and `checkpoint_all` are complete and
      six call sites reject requests once the flag is set, but nothing calls either: `axum::serve`
      runs without `with_graceful_shutdown` and there is no signal handler
- [ ] authentication and rate limiting. `ErrorKind::Unauthenticated` maps to 401 and nothing emits
      it; CORS allows any origin, method and header; `/config/reload` is unauthenticated
- [ ] one HTTP-level test that builds the router and hits it. 25 routes, no test constructs one
- [ ] test `apps/cli`. It has none, and `support-bundle` must never emit a secret
- [ ] migrate bincode 1.x → 2.x: a format migration for records, sidecars and the WAL, with a read
      path for existing data (RUSTSEC-2025-0141, ignored in `deny.toml` until then)
- [ ] migrate off `serde_yaml` 0.9, archived upstream. It parses every config file and the support
      bundle; `serde_norway` is the maintained fork
- [ ] implement `QuantizationLevel::Int4` and `Float16`, or drop the variants. `validate` rejects
      both today

## Next (v0.4.0) — the integrated baseline

One model, one GPU, batch size one, no HTTP.

- [ ] the interop spike, before the rest of this section: allocate with `hardware::gpu`, hand the
      pointer to candle as a tensor, run a kernel, verify numerics, and confirm in Nsight that no
      host round trip occurred
- [ ] pin `cudarc` to the version candle uses. Two versions produce two unrelated `CudaDevice`
      types and possibly two contexts, and shared-device operations route through host memory
- [ ] `candle` behind `inference-candle`, weights on the same device retrieval uses. Qwen is the
      supported model
- [ ] a forward-pass driver with `RetrievalHook` call sites from the first commit
- [ ] the first real `RetrievalHook` implementation, as its own crate
- [ ] an end-to-end benchmark — embed, search, top-k, fetch, tokenize, prefill, decode — reporting
      TTFT, tokens/sec, p50/p95 and recall
- [ ] measure four configurations against it: split process; in-process CPU index; in-process
      device-resident index; retrieval overlapped with prefill on its own stream
- [ ] measure retrieval frequency. Per-block at ~32 tokens is ~16 calls in a 512-token generation;
      per-layer is 32x that and needs sub-100µs queries. Which regime v0.6 targets sets the ANN
      requirement
- [ ] publish the result. Retrieval before prefill is the control arm
- [ ] spans on kernel launches and forward-pass stages

## Next (v0.5.0) — `piramid serve`

Co-located RAG with unmodified models. Also the baseline v0.6 is measured against.

- [ ] `piramid serve --model … --collection … --device cuda:0`
- [ ] `/api/infer` and an OpenAI-compatible `/v1/chat/completions`
- [ ] SSE streaming
- [ ] paged KV cache
- [ ] continuous batching
- [ ] an in-process embedding provider, `provider: piramid`, reusing the device retrieval already
      holds — encoder-only, no KV cache or sampling. A fourth option beside `openai` (including any
      server speaking that format) and `ollama`. It needs its own crate, since `embeddings` must
      not depend on `inference`; the binary wires it in through `EmbeddingsManager`, the same shape
      as a `RetrievalHook` implementation
- [ ] cut the website copy down to what the runtime does by then

## Later (v0.6.0) — retrieval during generation

Qwen is forked and hooked; other architectures use prompt-stuffing until adapters exist.

- [ ] fork the Qwen model implementation and add `RetrievalPoint` call sites between decoder
      layers. `forward` runs every layer internally, so this means one model file per architecture
- [ ] retrieval on its own CUDA stream, joined against the model's. Candle does not
      expose per-op stream control
- [ ] retrieval at block boundaries, measured against the v0.5 prompt-stuffed baseline at equal
      token budget
- [ ] retrieved documents pre-tokenized, so retrieval skips tokenization on the hot path
- [ ] precomputed document KV states reused at prefill. Document KV is not context-independent:
      position and preceding context change it, so states cannot be concatenated directly. See
      CacheBlend on selective recomputation
- [ ] a scheduler dividing VRAM and bandwidth between index, weights and KV under load

## Later (v0.7.0+) — beyond one model

- [ ] a trained adapter so fusion works on models Piramid has not forked
- [ ] fused kernels combining retrieval encoding and attention in one launch, once profiling shows
      launch or synchronisation overhead is material
- [ ] an index co-designed for the attention access pattern
- [ ] fp16/bf16 for weights and stored vectors, no upcasting on the hot path
- [ ] retrieval over the model's own KV history rather than external documents — long-context
      attention rather than document RAG, kept as its own branch. See RetrievalAttention
- [ ] retrieval at block boundaries under block-diffusion decoding, where a block is the retrieval
      unit instead of a token. Requires the autoregressive baseline for attribution

## Unscheduled

- [ ] backfill doc comments so `missing_docs` can move from `allow` to `warn` (~860 items)
- [ ] `piramid write-config` emits `Config::default()` rather than the commented
      `config.example.yaml`, so a generated file has the values without the explanations
- [ ] make `runtime:` reach further. A reload applies to collections opened after it, so a running
      collection keeps the search, cache and WAL settings it opened with. Either re-read them per
      request or document that it is per-collection-open
- [ ] no test starts the server and reloads config. The startup-block guard and the env-override
      precedence are covered at the loader level only
- [ ] untrack `apps/sdk/python/dist/` and `piramid.egg-info/` — build artifacts committed to git

## Out of scope

- a managed cloud service
- non-NVIDIA GPU backends until the CUDA path is real
- a second deployable process — see [decisions](decisions/README.md)
- distributed placement, and `cluster` beyond the local no-op router, until the single-device
  numbers exist
- vendor telemetry integrations — see [decisions](decisions/README.md)
- the npm and python SDKs, to be unpublished. They predate the one-shape request bodies and
  nothing depends on them
- growing the vector database past what the device path uses. Storage, the WAL and collections
  stay at maintenance
