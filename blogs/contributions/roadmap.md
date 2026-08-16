# Roadmap

This is the working roadmap for contributors. If you want to help, start here and pick one scoped task. If your idea is not listed but adjacent, open an issue first and propose it before implementation.

---

## Phase 1 — Retrieval-Fused Inference (v0.3.0)

**Chunked Cross-Attention (the fusion layer):**

- [ ] implement RETRO-style chunked cross-attention in Candle: retrieved neighbor chunks encoded by a shallow retrieval encoder, then attended to by decoder layers via cross-attention
- [ ] implement the retrieval encoder (2-layer bidirectional transformer) that encodes retrieved neighbors into dense representations for cross-attention
- [ ] wire the retrieval trigger: at each chunk boundary during the forward pass, query the index, encode neighbors, feed into cross-attention
- [ ] benchmark fusion vs naive prompt-stuffing on Natural Questions and TriviaQA — latency, recall, token cost

**RETROfit Training:**

- [ ] implement the retrofit finetuning loop: freeze base model weights, train only cross-attention + retrieval encoder params (~10% of total)
- [ ] retrofit Qwen 1.5B as the first target model — validate that it converges with retrieval on a standard QA dataset
- [ ] retrofit a second model (Llama 3B or Qwen 3B) to prove the pipeline generalizes
- [ ] publish retrofitted weights to HuggingFace as safetensors

**Serving:**

- [ ] `piramid pull <model>` — download a pre-retrofitted model
- [ ] `piramid serve --model qwen-1.5b-retro --data-dir ./data` — serve retrieval-fused inference
- [ ] add streaming token output (SSE)
- [ ] add OpenAI-compatible `/v1/chat/completions` endpoint that runs retrieval-fused inference transparently

---

## Phase 2 — GPU Kernels + Custom Indexing (v0.4.0)

**Custom CUDA Kernels:**

- [ ] define GPU backend struct and traits (cudarc)
- [ ] fused cross-attention kernel: combine retrieval encoding + cross-attention into a single kernel launch, eliminate intermediate materializations
- [ ] benchmark fused kernel vs naive Candle ops — target 2-3x speedup on the cross-attention step
- [ ] GPU-accelerated ANN search (distance computation on device, avoid CPU-GPU transfer on the hot path)

**Custom Indexing:**

- [ ] design Piramid's indexing algorithm co-optimized for the cross-attention access pattern — the index isn't just returning top-k, it's feeding an attention layer, so relevance scoring can be jointly learned
- [ ] experiment with learned index routing: the retrieval encoder's query representation selects index partitions, not a separate ANN lookup
- [ ] benchmark against standard HNSW/IVF on retrieval-fused inference quality (not just recall@k — end-to-end answer quality)

**Quantization (for inference, not just storage):**

- [ ] fp16/bf16 for both model weights and stored vectors — no upcasting on the hot path
- [ ] wire existing PQ implementation into search
- [ ] add binary quantization for candidate pre-filtering before full-precision reranking

---

## Phase 3 — Piramid Models (v0.5.0)

**Model Zoo:**

- [ ] publish 1.5B, 3B, 7B retrofitted models on HuggingFace
- [ ] each model co-optimized with Piramid's indexing algorithm
- [ ] benchmark suite: Natural Questions, TriviaQA, HotpotQA — compare against naive RAG (same base model + Qdrant + prompt stuffing)
- [ ] add model cards documenting training data, retrofit procedure, and known limitations

**Training Pipeline:**

- [ ] add dataset generation workflows for retrofit finetuning on custom domains
- [ ] `piramid retrofit <base-model> --data-dir ./data` — let users retrofit their own base model against their own index
- [ ] document compute requirements per model size (single GPU feasibility matrix)

---

## Phase 4 — Production Hardening (v0.6.0)

**Crash Safety & Durability:**

- [ ] WAL version fields and checksums; handle partial writes and format mismatches
- [ ] automatic index rebuild from WAL on detected corruption
- [ ] dry-run config validation on startup

**Write Path:**

- [ ] async write pipeline via `tokio-fs`: batching, buffered writes, background flush
- [ ] prefetching for sequential reads
- [ ] background job queue for long-running storage operations

**Bugs to Fix (blocking stability, not features):**

- [ ] remove the HNSW vector cache eviction bug: make delete/update graph semantics explicit (tombstone or rebuild)
- [ ] remove or redesign the metadata cache: filtered search and re-ranking need one explicit consistency model
- [ ] fix the embedding cache blocking mutex in async handlers — use async-aware lock or restructure
- [ ] fix read endpoints silently creating empty collections instead of returning 404

---

## Phase 5 — Retrieval Pipeline (v0.7.0)

**Hybrid Retrieval:**

- [ ] sparse/BM25 indexes alongside dense vectors
- [ ] reranking mechanisms (cross-encoder, ColBERT-style late interaction)
- [ ] context-packing policies: max tokens, diversity, source caps, recency weighting, metadata constraints

**Evaluation:**

- [ ] end-to-end RAG benchmarks: Piramid (retrieval-fused) vs baseline stack (vector DB + BM25 + reranker + prompt-stuffed LLM)
- [ ] retrieval recall, answer faithfulness, citation correctness, latency, memory, cost per query
- [ ] publish benchmark profiles for consumer hardware

**Filter & Query Features:**

- [ ] metadata indexing for fast pre-filtering
- [ ] range queries, regex matching, date filters, array membership, boolean filters (AND/OR/NOT)
- [ ] metadata-only search (no vector similarity)
- [ ] query result caching (LRU, TTL-based)

---

## Phase 6 — Distributed (v1.0.0)

**Cluster Runtime:**

- [ ] node runtime abstraction: stable IDs, capabilities, heartbeat, graceful shutdown
- [ ] static cluster membership for small trusted deployments first
- [ ] hardware-aware placement: CPU-only, iGPU, discrete GPU, storage-heavy, mixed profiles
- [ ] request routing: local by default, cross-network only when latency budget justifies it

**Distributed Search & Inference:**

- [ ] shard collections by vector ID or partition key with single-node fallback
- [ ] fan-out search with top-k merge, timeout budgets, partial-result reporting
- [ ] distributed inference routing: retrieve locally or remotely, pack context, route to best inference node, stream back
- [ ] continuous batching across clients per inference node with admission control
- [ ] KV-cache locality: route follow-up turns to the node that owns the session cache

**Observability:**

- [ ] distributed tracing across retrieve → rerank → context-pack → prefill → decode → stream
- [ ] cluster metrics: node health, shard ownership, queue depth, GPU memory, KV-cache usage
- [ ] failure-mode tests: node loss, slow shard, stale replica, interrupted stream, model overload

---

## Ongoing — Docs, SDKs, Platform

- [ ] separate API docs to `docs.piramiddb.com`
- [ ] document Rust SDK with examples
- [ ] publish Python SDK to PyPI
- [ ] add architecture note: Piramid is database-as-inference-memory, not model-weight decompilation
- [ ] add research log for failed fusion experiments so contributors know dead ends
- [ ] `piramid init` auto-detects system resources and configures accordingly
- [ ] hardware profiles (`8gb`, `16gb`, `32gb`, `cpu-only`, `gpu`) auto-select index type, quantization, cache size, search depth
