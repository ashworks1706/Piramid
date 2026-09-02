# piramid-index

ANN index implementations: flat, HNSW, IVF.

Indexes own traversal structure, not the vectors themselves. Those arrive through
`VectorReader`, so the same index works over a cache-backed, slab-backed, or device-resident store.

`IndexSearchRequest` is a struct rather than a parameter list, so new fields don't break every
implementation.

IVF is the device family: scan `nprobe` clusters, fully parallel. HNSW is the host family; graph
traversal parallelises poorly on a GPU, which is why FAISS-GPU ships IVF-Flat and IVF-PQ and leaves
HNSW on the CPU. Flat is the reference both are measured against.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
