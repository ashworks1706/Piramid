# 0005 — Contiguous vector layout behind an optional seam

**Context.** Vectors are stored as `HashMap<Uuid, Vec<f32>>` in the cache and the default
`VectorReader`. Every vector is a separate heap allocation, scattered across the address space.
`VectorReader` exposed only `get(&Uuid) -> Option<&[f32]>` — one vector at a time.

This has two costs. The prefetcher cannot stride across candidates, so the SIMD path underperforms.
And a device backend must gather N scattered rows into a staging buffer on every call, which
generally costs more than the kernel saves — the GPU path would measure *slower* than CPU, for
reasons invisible in the kernel.

**Decision.** Introduce `VectorSlab`: one `Vec<f32>` with a fixed stride and a `Uuid → u32`
ordinal map, so a candidate set is a subslice.

Widen `VectorReader` with `as_slab() -> Option<(&[f32], usize)>` and `gather_into()`, **both with
default implementations**. The scattered reader returns `None` from `as_slab` rather than silently
copying, because hiding that cost would make the CPU/device choice unmeasurable.

**Consequences.** The seam exists with zero behavior change: every existing reader satisfies the
trait unchanged, and call sites migrate one at a time. `DistanceKernels`' batch methods take a slab
for exactly this reason — the two decisions are the same decision seen from either end.

Nothing is migrated onto it yet. That is deliberate: the layout change and the structural change
should not land in the same commit.

**Not decided.** Whether the slab is the storage format or a projection built on load, and whether
ordinals become the primary key inside index structures — a `u32` in an HNSW adjacency list instead
of a 16-byte `Uuid` is a large win, but it changes the sidecar format.
