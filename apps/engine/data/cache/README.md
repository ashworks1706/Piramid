# piramid-cache

Per-collection in-memory state, split honestly: `VectorStore` is resident — the ANN indexes
resolve candidate ids through it, so evicting an entry breaks search — and `MetadataCache` is a
real cache, bounded and safe to drop. `CacheManager` owns both and is the one place a new
per-collection cache gets added. See ADR 0012.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
