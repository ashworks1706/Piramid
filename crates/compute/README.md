# piramid-compute

Distance and similarity kernels with CPU/GPU backend dispatch.

A leaf crate: it depends on nothing else in the workspace, so kernels can be benchmarked in
isolation.

Adding a backend is one file implementing `DistanceKernels` plus one arm in the registry. Batch
methods take a contiguous row-major slab and a caller-owned `out`, because that shape uploads to a
device in one copy — a slice of `Vec`s cannot, and forces a per-call gather that costs more than
the kernel saves.

Dispatch never panics: a requested-but-unavailable backend falls back to CPU with a warning.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md) for how the crates fit together.
