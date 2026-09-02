# piramid-inference

Model execution and the seam where retrieval enters the forward pass.

Scaffolding. Every module is a boundary with its contract written down and nothing behind it
yet.

The piece worth getting right early is `augment::RetrievalHook`. It's defined before anything can
call it because a forward-pass driver written without the seam is hard to retrofit with one, and
it's mechanism-agnostic on purpose. See `docs/decisions/0006-retrieval-fusion-seam.md`.

`HiddenState` is either a host slice or a `DeviceBuffer`, so fusing into device memory needs no
host round trip. `launch` is separate from `join` so retrieval can run on its own stream while the
model computes.

This crate depends on nothing in the retrieval stack. A strategy that actually queries an index is
a separate crate depending on both this one and `piramid-search`, which is what keeps a collection
queryable with no model loaded.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
