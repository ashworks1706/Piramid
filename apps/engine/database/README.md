# piramid-database

Where vectors live, how they are found, and the object that owns both.

`storage` is bytes: records, the write-ahead log, mmap, sidecar files, the offset index and the
manifest. A `RecordStore` does not know what a collection is.

`index` owns traversal structure and the sidecar format — flat, HNSW and IVF. Indexes do not own
the vectors; those arrive through `VectorReader`, so the same index works over a cache-backed,
slab-backed or device-resident store. The families are not interchangeable across hardware: IVF is
the device family, HNSW the host family, and flat the reference both are measured against.

`search` owns what a query asks for — overfetch planning, filtering, scoring, ranking, and the
near-duplicate scan that is the same work with every stored vector as the query.

`Collection` composes them: a record store, two caches, a checkpoint policy and an index, behind
one queryable object.

These share a crate because separating them is a cycle. A collection is built on search, search is
built on storage, and storage is where a collection's bytes live — so any split that puts the
collection above and the bytes below has the middle depending on both ends.

`unsafe` appears once, at `storage::sidecars::mmap::create_mmap`, with a `// SAFETY:` note.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
