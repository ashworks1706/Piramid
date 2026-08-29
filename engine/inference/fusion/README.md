# piramid-fusion

The retrieval-fusion seam: where retrieval enters a model's forward pass.

One trait, `RetrievalHook`, plus the types it needs. Depends on `piramid-core` and nothing else, so
the forward-pass driver can hold the hook call sites without depending on the retrieval stack —
a concrete fusion strategy is a separate crate that depends on both.

Mechanism-agnostic on purpose. See `docs/decisions/0006-retrieval-fusion-seam.md`.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
