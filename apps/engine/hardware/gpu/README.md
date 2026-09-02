# piramid-gpu

Device runtime: contexts, buffers, streams, and compiled kernels.

Owns talking to a device and nothing about what the math means. Vendor SDK types stay inside
`backends/`; everything above sees only `Device`, `DeviceBuffer`, `Stream`, and `KernelModule`.

A leaf crate on purpose. Both `piramid-compute` for distance kernels and `piramid-inference` for
model execution need a device, and neither should depend on the other to get one. Sharing a single
`Device` is what puts vectors and model weights in the same address space.

This is the one crate where `unsafe` is expected. Every block carries a `// SAFETY:` note.

One constraint on that leaf property, worth knowing before it bites: once `piramid-inference` loads
candle, this crate's `cudarc` version has to match candle's exactly. Two versions means two
unrelated `CudaDevice` types and possibly two contexts on one card, and every "shared device"
operation silently routes through host memory instead — which is the one thing the whole design
exists to avoid. So `gpu` stays a leaf in the dependency graph while its vendor pin is dictated
from above.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
