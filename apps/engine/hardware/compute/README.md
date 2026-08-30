# piramid-compute

Distance and similarity kernels with CPU and GPU backend dispatch, plus the quantization
encodings the kernels will score (`quantization::QuantizedVector` and its config).

A leaf crate: it depends on nothing else in the workspace, so kernels can be benchmarked on
their own.

Adding a backend is one file implementing `DistanceKernels` plus one arm in the registry. The batch
methods take a contiguous row-major slab and a caller-owned `out`, because that shape uploads to a
device in one copy. A slice of `Vec`s can't, and forces a per-call gather that costs more than the
kernel saves.

Dispatch never panics. A requested backend that isn't available falls back to CPU with a warning.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
