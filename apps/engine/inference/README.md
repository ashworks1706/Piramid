# piramid-inference

Model execution and the seam where retrieval enters the forward pass.

Scaffolding. Every module is a boundary with its contract written down and nothing behind it
yet.

The piece worth getting right early is `augment::RetrievalHook`. It's defined before anything can
call it because a forward-pass driver written without the seam is hard to retrofit with one, and
it's mechanism-agnostic on purpose. See `docs/decisions/0006-retrieval-fusion-seam.md`.

Two things in its shape are load-bearing, and both exist because retrieval and inference are meant
to share a device (`docs/decisions/0015-the-retrieval-seam-is-device-aware-and-split.md`).
`HiddenState` is either a host slice or a `DeviceBuffer`, since a host-only seam would force a
device-to-host-to-device copy per invocation — per layer, at `LayerEntry`. And `launch` is split
from `join` so retrieval can overlap model compute on its own stream; one fused call serialises
them however it is implemented.

This crate depends on nothing in the retrieval stack. A strategy that actually queries an index is
a separate crate depending on both this one and `piramid-search`, which is what keeps a collection
queryable with no model loaded.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
