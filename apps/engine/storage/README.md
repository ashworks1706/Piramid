# piramid-storage

Records, WAL, sidecars, mmap, and vector layout. `SidecarManager` owns every sidecar
path and format beside a record file.

Byte-level primitives for the domain layer above. Storage decides nothing about API behaviour,
search semantics, or collection lifecycle.

`VectorReader::as_slab` is the seam for a contiguous layout: a reader stored that way returns its
buffer and a batch kernel or device upload takes it in one copy. Nothing implements it yet — the
contiguous store is a v0.3.0 roadmap item — so every reader falls back to `gather_into`.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
