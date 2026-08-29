# piramid-storage

Records, WAL, sidecars, mmap, vector layout, and quantization.

Byte-level primitives for the domain layer above. Storage decides nothing about API behaviour,
search semantics, or collection lifecycle.

`vectors::VectorSlab` is the contiguous layout that makes SIMD and device upload viable.
`VectorReader::as_slab` is the seam for adopting it one call site at a time.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
