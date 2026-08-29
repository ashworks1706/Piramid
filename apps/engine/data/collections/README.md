# piramid-collections

The collection domain object: lifecycle, caching, checkpointing, compaction.

Where storage, index, cache, and search meet. A collection is the unit that owns all four.

The record store plus sidecars are the source of truth; cache and index are acceleration
structures that must stay rebuildable from stored records.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
