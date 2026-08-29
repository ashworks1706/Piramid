# piramid-inference

Model execution and retrieval fusion.

Scaffolding: every module is a boundary with its contract written down and no implementation
behind it.

The piece that matters is `fusion::RetrievalHook`, the seam where retrieval enters the forward
pass. It is defined before anything can call it because a forward-pass driver written without the
seam is very hard to retrofit with one. It is mechanism-agnostic on purpose — see
`docs/decisions/0006-retrieval-fusion-seam.md`.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
