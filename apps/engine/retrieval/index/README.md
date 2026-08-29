# piramid-index

ANN index implementations: flat, HNSW, IVF.

Indexes own traversal structure, not the vectors themselves — those arrive through
`VectorReader`, so the same index works over a cache-backed, slab-backed, or device-resident
store.

`IndexSearchRequest` is a struct rather than a parameter list so new fields do not break every
implementation.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
