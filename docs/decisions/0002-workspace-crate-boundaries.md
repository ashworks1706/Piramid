# 0002 — Split the crate into a workspace

> Folder naming superseded by [0008](0008-engine-and-apps.md). The crate boundaries and the
> dependency law established here are unchanged.


**Context.** The codebase was a single library with nineteen modules. The module dependency graph
contained six cycles — `config` ↔ `index`, `storage` ↔ `collections`, `index` ↔ `search`, among
others — and `compute` had grown a dependency on `config`. Nothing prevented a layer from reaching
upward; the old `docs/architecture.md` described a layering that the compiler did not check, and it had
already eroded.

**Decision.** Eleven crates, one per ownership boundary, with the dependency law recorded in
`docs/ARCHITECTURE.md` and enforced by `scripts/check-deps.sh` in CI and the pre-commit hook.

Folder names are plain (`crates/compute`, `crates/collections`); package names are prefixed
(`piramid-compute`) so they are publishable without squatting generic crates.io names.

Breaking the cycles moved each type to the layer that owns it: `EmbeddingConfig` and `IndexConfig`
to `core::config`, index sidecar persistence to `index`, `Filter` to `core::metadata` beside the
`Metadata` it matches on, and `search::engine` onto a `SearchTarget` so it no longer needs to know
what a `Collection` is.

**Consequences.** An upward edge is now a compile error, and a *declared* upward edge is a CI
failure. Touching a kernel no longer rebuilds the HTTP server. GPU dependencies scope to one crate.
Each layer gets its own tests and benches.

Costs: eleven manifests, `pub(crate)` widening to `pub` at former module boundaries, and
dependency-ordered publishing — `release-plz` handles the ordering.

**Not decided.** Whether `cluster` stays inside `server`. It is 103 lines that always route
locally; promoting it to a crate before it does anything would be scaffolding for its own sake.
