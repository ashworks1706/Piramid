# piramid-collections

The Collection object: lifecycle, caching, checkpointing, compaction.

`cache::VectorStore` is resident — the ANN indexes resolve candidate ids through it, so evicting an
entry breaks search. `cache::MetadataCache` is a real cache, bounded and safe to drop.

Where storage, cache, index, and search meet. A collection is the unit that owns all four.

The record store plus sidecars are the source of truth. Cache and index are acceleration
structures that have to stay rebuildable from stored records.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
