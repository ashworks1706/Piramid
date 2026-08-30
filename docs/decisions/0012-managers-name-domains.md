# 0012 — Managers name domains, not resources

**Context.** Coming back to the tree cold, the question "where do I add a new cached thing?" had
no single answer: `CacheManager` lived inside `collections`, sidecar paths were `format!` calls
scattered across four files (one call site aliased the imports to make them readable), and
quantization sat in `storage` while its 1-bit sibling lived in `compute::backends::binary`.
`CacheManager` also held two things under one name: a resident vector map that must never be
evicted — indexes resolve candidate ids through it, so eviction breaks search — and a bounded
metadata cache that exists to be dropped. Two tests already asserted that difference; the type
didn't.

**Decision.** Each domain gets one entry point that owns its state, and the entry point is a
`*Manager` only when there is state to own. Resources a manager hands out keep their specific
names — `Device`, `Wal`, `RecordStore`, `Collection` — because "manager" says less than the noun
it would replace, and `AppState` stays `AppState` because it genuinely is shared state.

- `collections::cache` is the caching domain. `VectorStore` is the resident map (a store, stated
  as one); `MetadataCache` is the bounded cache; `CacheManager` owns both. Anything new that
  caches per-collection state becomes a field there, not a static elsewhere. It lives inside
  `collections` rather than as its own crate because a collection is its only consumer — and a
  cache belongs to the domain whose data it caches, which is also why `CachedEmbedder` stays in
  `embeddings` and the KV cache will stay in `inference`.
- `SidecarManager` in `storage::persistence` owns every sidecar path and format beside a record
  file — offsets, manifest, WAL, WAL meta, and the ANN index's *location* (`piramid-index` still
  owns that file's format). A new sidecar is a new method, never a `format!` at a call site.
- `quantization` moves to `compute` as a module with its config, beside the binary kernel that is
  already quantization. The missing piece that kept PQ unreachable is a *distance* kernel — PQ's
  point is scoring a full-precision query against codes without decompressing — and that kernel
  can only live in `compute`, so the format now sits next to it. `core::config` re-exports the
  config types the same way it re-exports `ExecutionMode`. Corrupt encodings are `ComputeError`s.
- `GpuManager` (device acquisition, future multi-device policy) and `InferenceManager` (model,
  KV cache, batch queue, the retrieval hook) are scaffolded ahead of their code, per the house
  rule, so drivers get written against a surface instead of growing their own.
- `EmbeddingsManager` owns the embedder stack and its throughput counters, which were two
  separate `AppState` fields; building the stack from config moved out of the binary into
  `EmbeddingsManager::from_config`, and the two duplicated `AppState` constructors collapsed
  into one that takes the manager.

The rule is one manager per *crate with state*, in its `manager.rs`, importable from the crate
root — never defined in `lib.rs`, which stays a re-exporting table of contents. Grouping folders
(`hardware/`, `data/`, `retrieval/`) get nothing: they are navigation, not compilation units, and
cannot own state. The thing that composes the retrieval crates at runtime is `Collection`.

**Not done.** No manager for stateless modules — `compute`'s kernels, `validation`,
`services::convert`. A struct with no fields is a place people hang state, and for `compute`
specifically that is how the leaf property would erode. No `MetricManager`: counters are
decentralized plain atomics on purpose, so recording never takes a lock or links an exporter;
the Prometheus `Registry` already centralizes at scrape time. No renaming `ObservabilityGuard`
to a manager — a guard is held, not called, and the name carries the one contract that matters
(drop it early and spans are lost), matching `tracing-appender`'s `WorkerGuard`. `AppState`
is the composition root that holds the managers, which is a different job from being one. No generic cache abstraction over
`piramid-cache`, `embeddings::CachedEmbedder`, and the future KV cache: they share a word, not a
shape (different keys, values, bounds, and eviction rules).

**Consequences.** No new crates; `collections` gains a `cache/` module. `storage` no longer
mentions quantization. The umbrella crate re-exports every manager in one block, so
`use piramid::CacheManager` works and the moving parts have one address.
Sidecar path knowledge has one home. The roadmap's slab migration now names its real target:
`cache::VectorStore`, the half of the old `CacheManager` that was never a cache.
