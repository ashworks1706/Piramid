# piramid-retrieval

How vectors are found.

`index` owns traversal structure and the sidecar format — flat, HNSW and IVF. Indexes do not own
the vectors themselves; those arrive through `VectorReader`, so the same index works over a
cache-backed, slab-backed or device-resident store.

`search` owns what a query asks for: overfetch planning, filtering, scoring and ranking. It never
learns what a collection is.

The families are not interchangeable across hardware. IVF is the device family: scan `nprobe`
clusters, fully parallel. HNSW is the host family; graph traversal parallelises poorly on a GPU,
which is why FAISS-GPU ships IVF-Flat and IVF-PQ and leaves HNSW on the CPU. Flat is the reference
both are measured against.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
