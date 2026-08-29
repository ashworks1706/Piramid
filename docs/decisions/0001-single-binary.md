# 0001 — One binary, layers as crates

**Context.** Piramid's pitch is that retrieval and inference share a process: no network hop
between the index and the model, and eventually one device address space holding both vectors and
weights. Comparable systems are assembled from separate services (a vector database, an inference
server, glue). We are deliberately not that.

**Decision.** Ship exactly one deployable, the `piramid` binary. Internal layers are library
crates under `crates/`, not services. `apps/` holds deployables and currently contains only the
CLI.

**Consequences.** No inter-service protocol to version, no serialization between retrieval and
inference, and the single-address-space claim stays true. In exchange, layering has no network to
enforce it, which is why [0002](0002-workspace-crate-boundaries.md) makes it physical instead.

**Not decided.** Whether a future distributed mode splits anything into a second process.
`crates/server/src/cluster` is a routing boundary that always answers `Local`; distribution is a
placement problem, and placement is easier to reason about once inference exists and dictates where
things must live.
