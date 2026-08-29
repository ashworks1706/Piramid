# Piramid — agent guide

An inference-native retrieval engine in Rust: vector storage, ANN search, and (eventually)
transformer inference in one process. Read `docs/ARCHITECTURE.md` for crate boundaries and
invariants, `docs/ROADMAP.md` for what we're building and in what order. Do not contradict either
— propose an edit to the doc instead.

## Commands

`just` is the entrypoint (`just` lists recipes). Install: https://just.systems.

```
just doctor | env | hooks | bootstrap   # first run
just check              # the gate: fmt, clippy, tests, layering, website
just check-rust         # cargo fmt --check, clippy -D warnings, test, scripts/check-deps.sh
just check-features     # compile-check --features gpu-cuda and inference-candle
just fmt                # format every unit in place
just serve              # run the server on :6333
just cli show config    # any CLI subcommand
just cli support-bundle # diagnostics to attach to a bug report
just doc                # rustdoc, warnings are errors
just bench              # criterion
just audit              # cargo-deny: advisories, bans, licences, sources
just up | down | logs   # docker compose
```

A change is not done until `just check` passes. CI and the pre-commit hook run the same recipes.

## Layout

One repo, one binary. Library layers are `apps/engine/`; the only deployable is `apps/cli`.
Language is never a folder. Hardware is never a folder.

```
apps/engine/core                  errors, config, metadata + filters, validation, telemetry
apps/engine/hardware/compute      distance kernels + backend registry    (leaf: no workspace deps)
apps/engine/hardware/gpu          device, buffer, stream, module, kernels (leaf: no workspace deps)
apps/engine/data/storage          records, WAL, sidecars, mmap, VectorSlab, quantization
apps/engine/data/collections      Collection domain object, cache, checkpoint, compact
apps/engine/retrieval/index       flat, hnsw, ivf, selector, sidecar persistence
apps/engine/retrieval/search      query planning, filtering, scoring, ranking
apps/engine/retrieval/embeddings  openai, ollama, local providers
apps/engine/inference             model, forward pass, kv_cache, batching, sampling,
                                  and retrieval/ — the RetrievalHook seam
apps/engine/service/server        http, services, runtime state, cluster
apps/engine/service/observability tracing subscriber, OTLP, Sentry, Prometheus rendering
apps/cli                          the `piramid` binary + the umbrella `piramid` facade crate
apps/website                      piramiddb.com, blog content and images included
apps/sdk                          npm and python clients
docs/                             ARCHITECTURE.md, ROADMAP.md, decisions/
deploy/                           compose + one Dockerfile per image
```

Groups say what a thing is for: `hardware/` changes when the machine changes, `data/` is where
vectors live and who owns them, `retrieval/` is how you find them, `inference/` how you run a model
over them, `service/` how the outside world reaches it. They are navigation, not dependency order —
the law below is the authority on direction.

## The dependency law

A crate may depend on one listed below it. The reverse is a layering violation.

```
compute ─┐                    gpu ─┐
         │                         ├─→ inference ─┐
core ────┼─→ storage ─→ index ─→ search ─→ collections ─→ server ─→ cli
         └─→ embeddings ──────────────────────────────────┘
```

Enforced by `scripts/check-deps.sh`, which runs in `just check-rust`, the pre-commit hook, and CI.
Adding an edge means editing that script *and* `docs/ARCHITECTURE.md` in the same change.

**`compute` and `gpu` depend on nothing in the workspace**, including `core`. That is what lets
kernels be lifted into a standalone benchmark, and what stops `inference` reaching through
retrieval math to get at a device.

## The three seams

Everything else is infrastructure for these. Change them deliberately.

- **`compute::DistanceKernels`** — one backend per file in `compute/backends/`, one arm in the
  registry. Batch methods take a *contiguous row-major slab* and a caller-owned `out`, because
  that shape uploads to a device in one copy. `&[Vec<f32>]` does not; never reintroduce it.
- **`storage::vectors::VectorReader`** — how indexes read vectors they do not own.
  `as_slab()` is the fast path, `gather_into()` the fallback. Both have defaults, so a new reader
  costs nothing.
- **`inference::retrieval::RetrievalHook`** — where retrieval enters the forward pass. Defined
  before anything can call it, on purpose: a driver written without the seam is very hard to
  retrofit with one. A strategy that queries an index is its own crate — `inference` must never
  depend on the retrieval stack.

## Rules

- Workspace lints are the law (`[workspace.lints]` in the root `Cargo.toml`). No `panic!`,
  `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!` outside `apps/cli`. Fix at the source
  — never `#[allow]` a lint to get green. A genuine exception gets the narrowest possible scope
  and a one-line reason.
- `unsafe_code` is denied workspace-wide. It is allowed in `apps/engine/hardware/gpu` (device memory) and at
  exactly two audited sites: `storage::persistence::mmap::create_mmap` and
  `server::runtime::disk`. Every block carries a `// SAFETY:` comment stating the precondition.
- **A library never ends the process.** No `std::process::exit` outside `apps/cli`. Loading
  configuration returns `Result`; the binary decides what to do with it.
- **Core is transport-agnostic.** `PiramidError` exposes `ErrorKind`, never a `StatusCode`.
  HTTP mapping lives in `server::http::ApiError`.
- Vendor SDK types stay inside their backend module — `gpu/backends/`, `inference/backends/`.
  Nothing above imports `cudarc` or `candle`.
- Dependencies are declared in `[workspace.dependencies]` and referenced with `.workspace = true`.
- Errors: `thiserror` enums, `Result` aliases per layer, no `unwrap` outside tests.
- Logging: `tracing` with structured fields and a `target:`, never `println!`.
- Feature flags are additive and default-off. `cargo build` must always work with no CUDA toolkit.

## Conventions

- Unit tests next to the code (`#[cfg(test)] mod tests`); integration tests in `apps/engine/<group>/<crate>/tests/`.
- Every public item has a `///` doc comment saying *what*, not *how*. Every module has `//!`.
- Traits are named for the capability, not the implementation. Backends are named for the
  technology, one file each — new hardware is a new file, never a new match arm.
- `mod.rs` / `lib.rs` re-export; they do not define types.
- One canonical path per item. No module re-exports another module's contents.
- The tree is scaffolded ahead of the code. Fill a stub in place; don't create parallel files or
  rename a stub without updating `docs/ARCHITECTURE.md`.
- Commit messages: imperative subject ≤ 72 chars, body explains *why*.
- A decision that changes a boundary gets an ADR in `docs/decisions/`.

## Skills

`.claude/skills/README.md` lists them. Use `rust-skills` when writing Rust,
`test-driven-development` for features and fixes, `systematic-debugging` for bugs,
`verification-before-completion` before claiming anything is done, `kernel-authoring` when adding
a compute backend or GPU kernel, `security-audit-standard` before a release, `/code-quality` for
cleanup passes, `/adr` to record a decision. `/check` is the gate.

## Out of scope

See "Out of scope" in `docs/ROADMAP.md`. Don't build toward those without an explicit decision.
