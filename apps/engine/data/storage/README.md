# piramid-storage

Persistence primitives: records, WAL, sidecars, mmap, vector layout, quantization.

Byte-level primitives for the domain layer above. Storage decides nothing about API behavior,
search semantics, or collection lifecycle.

`vectors::VectorSlab` is the contiguous layout that makes SIMD and device upload viable;
`VectorReader::as_slab` is the seam for adopting it incrementally.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
