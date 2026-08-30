# Decisions

One short file per decision: context, decision, consequences, and what was deliberately left open.
Named `NNNN-slug.md`.

A decision earns a record when it changes a boundary, forecloses an option, or will look arbitrary
to someone reading the tree in a year. Write it when the decision is made, not afterwards.

| # | Decision |
|---|---|
| [0001](0001-single-binary.md) | One binary, layers as crates |
| [0002](0002-workspace-crate-boundaries.md) | Split the crate into a workspace |
| [0003](0003-backend-first-compute-dispatch.md) | Backend-first compute, trait dispatch |
| [0004](0004-gpu-owns-device-compute-owns-math.md) | `gpu` owns the device, `compute` owns the math |
| [0005](0005-contiguous-vector-layout.md) | Contiguous vector layout behind an optional seam |
| [0006](0006-retrieval-fusion-seam.md) | Commit to the fusion seam, not to RETRO's mechanism |
| [0007](0007-transport-agnostic-errors.md) | Errors carry a kind, not a status code |
| [0008](0008-engine-and-apps.md) | `apps/engine/` for the library tree, `apps/` for what we ship |
| [0009](0009-retrieval-hook-is-a-module.md) | The retrieval hook is a module, not a crate |
| [0010](0010-name-audit.md) | One name, one meaning |
| [0011](0011-open-standards-only.md) | Telemetry speaks open standards only |
| [0012](0012-managers-name-domains.md) | Managers name domains, not resources |
| [0013](0013-strategies-are-not-backends.md) | Strategies are not backends |
