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

- `piramid-cache` is a crate under `data/`, the caching domain. `VectorStore` is the resident
  map (a store, stated as one); `MetadataCache` is the bounded cache; `CacheManager` owns both.
  Anything new that caches per-collection state becomes a field there, not a static elsewhere.
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

A manager's code lives in its domain's `manager.rs` — never in `lib.rs`, which stays a
re-exporting table of contents per the existing convention. The import path is the crate root
either way; the file location is for readers.

**Not done.** No manager for stateless modules — `compute`'s kernels, `validation`,
`services::convert`. A struct with no fields is a place people hang state, and for `compute`
specifically that is how the leaf property would erode. No generic cache abstraction over
`piramid-cache`, `embeddings::CachedEmbedder`, and the future KV cache: they share a word, not a
shape (different keys, values, bounds, and eviction rules).

**Consequences.** New crate `piramid-cache`; edges `cache → core`, `cache → storage`,
`collections → cache` (and the CLI facade re-export). `storage` no longer mentions quantization.
Sidecar path knowledge has one home. The roadmap's slab migration now names its real target:
`cache::VectorStore`, the half of the old `CacheManager` that was never a cache.
