# Piramid — agent guide

An inference-native retrieval engine in Rust. Read `docs/ARCHITECTURE.md` for crate boundaries and
invariants, `docs/ROADMAP.md` for what we're building and in what order. Don't contradict either;
propose an edit to the doc instead.

## Commands

`just` is the entrypoint for working on the repo. Run `just` on its own to list recipes. Install
from https://just.systems.

It is contributor tooling, not the product. Nothing shipped depends on it: the Dockerfiles call
cargo directly, and the published `piramid` binary knows nothing about it. When writing docs aimed
at someone *using* Piramid, show `piramid ...` or `docker ...`, never `just ...`.

```
just doctor | env | hooks | bootstrap   first run
just check              the gate: fmt, clippy, tests, layering, website
just check-rust         cargo fmt --check, clippy -D warnings, test, scripts/check-deps.sh
just check-features     compile-check --features gpu-cuda, inference-candle, otel
just fmt                format everything in place
just serve              run the server on :6333
just cli show config    any CLI subcommand
just cli support-bundle diagnostics to attach to a bug report
just doc                rustdoc, warnings are errors
just bench              criterion
just audit              cargo-deny: advisories, bans, licences, sources
just web                dev server for the site on :3000
just web-preview        build and serve what actually deploys
just web-shots          headless screenshots into target/screenshots
just up | down | logs   docker compose
```

A change isn't done until `just check` passes. CI and the pre-commit hook run the same recipes, so
local green means CI green.

CI only runs what the diff touches: a website change skips the Rust matrix and a kernel change
skips eslint. The `changes` job in `ci.yml` owns that mapping, so a new top-level directory needs
a filter added there or nothing will run for it.

## Layout

One repo, one binary. Everything we author is under `apps/`. The library crates are
`apps/engine/`; the only thing that ships as an executable is `apps/cli`. Language is never a
folder and neither is hardware.

```
apps/engine/core                  errors, config, metadata and filters, validation,
                                  stats (what the engine measures about itself)
apps/engine/observability         where those measurements go: subscriber, OTLP,
                                  Prometheus encoding
apps/engine/hardware/compute      distance kernels, backend registry, quantization encodings
apps/engine/hardware/gpu          device, buffer, stream, module, kernels
apps/engine/data/storage          records, WAL, SidecarManager, mmap, VectorReader
apps/engine/data/cache            VectorStore (resident), MetadataCache (bounded),
                                  CacheManager owning both
apps/engine/data/collections      the Collection object, checkpoint, compact
apps/engine/retrieval/index       flat, hnsw, ivf, selector, sidecar persistence
apps/engine/retrieval/search      query planning, filtering, scoring, ranking
apps/engine/retrieval/embeddings  openai (the wire format, local servers included), ollama
apps/engine/inference             forward pass, kv_cache, batching, sampling,
                                  augment (the RetrievalHook seam)
apps/engine/server                http (axum only), services (locks, metrics, DTOs),
                                  state, disk, cluster
apps/cli                          the piramid binary and the umbrella piramid facade crate
apps/website                      piramiddb.com, blog content and images included
apps/sdk                          npm and python clients
docs/                             ARCHITECTURE.md, ROADMAP.md, decisions/
deploy/                           compose and one Dockerfile per image
```

`core` and `observability` are flat because they're used from everywhere rather than sitting at
one level. `server` and `inference` are flat because each is a single crate. The grouped folders
say what they're for: `hardware/` is the code that changes when the machine changes, `data/` is
where vectors live and who owns them, `retrieval/` is how you find them. Groups are for finding
your way around; the dependency rule below is what actually constrains anything.

## The dependency rule

A crate may depend on one listed below it. The reverse is a layering violation.

```
compute ─┐                    gpu ─┐
         │                         ├─→ inference ─┐
core ────┼─→ storage ─→ index ─→ search ─→ collections ─→ server ─→ cli
         │        └────────→ cache ───────────┘   │
         └─→ embeddings ──────────────────────────┘
```

`scripts/check-deps.sh` enforces it, and runs in `just check-rust`, the pre-commit hook, and CI.
Adding an edge means editing that script and `docs/ARCHITECTURE.md` in the same change.

`compute` and `gpu` depend on nothing in the workspace, `core` included. That's what lets kernels
be benchmarked on their own, and what stops `inference` reaching through the retrieval math to get
at a device.

## The three seams

Everything else is infrastructure for these. Change them deliberately.

`compute::DistanceKernels` — one strategy per file in `compute/strategies/`, one arm in the registry.
"Backends" means the vendor layer and lives only in `gpu` and `inference`; see ADR 0013.
Batch methods take a contiguous row-major slab and a caller-owned `out`, because that shape
uploads to a device in one copy. A slice of `Vec`s can't, and forces a gather on every call that
costs more than the kernel saves. Don't reintroduce it.

`storage::vectors::VectorReader` — how indexes read vectors they don't own. `as_slab()` is the
fast path and `gather_into()` the fallback. Both have defaults, so a new reader costs nothing.

`inference::augment::RetrievalHook` — where retrieval enters the forward pass. Defined before
anything can call it, because a driver written without the seam is hard to retrofit with one. A
strategy that queries an index belongs in its own crate; `inference` must never depend on the
retrieval stack.

## Rules

- Workspace lints are enforced (`[workspace.lints]` in the root `Cargo.toml`). No `panic!`,
  `todo!`, `unimplemented!`, `dbg!`, `println!`, or `eprintln!` outside `apps/cli`. Fix at the
  source rather than adding an `#[allow]`. A real exception gets the narrowest possible scope and
  a one-line reason.
- `unsafe_code` is denied workspace-wide. It's allowed in `apps/engine/hardware/gpu` and at two
  audited sites, `storage::persistence::mmap::create_mmap` and `server::disk`. Every
  block carries a `// SAFETY:` comment stating its precondition, and the security workflow fails
  if a fourth site appears.
- A library never ends the process. No `std::process::exit` outside `apps/cli`. Loading
  configuration returns a `Result` and the binary decides what to do with it.
- `core` is transport-agnostic. `PiramidError` exposes an `ErrorKind`, never a `StatusCode`. HTTP
  mapping lives in `server::http::ApiError`.
- Vendor SDK types stay inside their backend module, `gpu/backends/` and `inference/backends/`.
  Nothing above imports `cudarc` or `candle`.
- Telemetry speaks open standards only. Prometheus and OTLP are protocols; a vendor's product is
  not. See ADR 0011.
- Dependencies go in `[workspace.dependencies]` and are referenced with `.workspace = true`.
- Errors are `thiserror` enums with a `Result` alias per layer. `unwrap_used` and `expect_used`
  are denied; test files opt back in with a module-level `#![allow]` and a reason.
- Logging is `tracing` with structured fields and a `target:`, never `println!`.
- Feature flags are additive and off by default. `cargo build` has to work with no CUDA toolkit.
- One way to do a thing. No fallback that answers when the thing asked for is unavailable, no
  second spelling of a name, no singular form of a plural request. If it cannot do what was asked,
  it returns an error saying so. A backend that quietly serves different numbers, a config knob
  nothing reads, and a `try_` wrapper around an infallible call are all the same bug.

## Conventions

- Unit tests next to the code in `#[cfg(test)] mod tests`, integration tests in the crate's
  `tests/`. Test data goes to `CARGO_TARGET_TMPDIR`, never a path relative to the crate.
- Every public item has a `///` comment saying what it is, not how it works. Every module has a
  `//!`.
- Comments explain why, never what. If a comment restates the line below it, delete it.
- One name, one meaning. Before naming a module, check the word isn't already used for something
  else in the tree. Repeating a word is fine when it means the same thing at each layer, as with
  `config/index.rs` and `error/index.rs`, and a problem when it doesn't. See ADR 0010.
- Traits are named for the capability, not the implementation. Strategies and backends are named
  for the technology, one file each, so new hardware is a new file rather than a new match arm.
- `mod.rs` and `lib.rs` re-export; they don't define types. A domain's manager lives in its
  `manager.rs` and is importable from the crate root, one predictable place per domain. Managers
  exist only for crates with state to own; grouping folders never get one.
- One canonical path per item. No module re-exports another module's contents.
- The tree is scaffolded ahead of the code. Fill a stub in place rather than creating a parallel
  file, and don't rename a stub without updating `docs/ARCHITECTURE.md`.
- Commit messages: imperative subject under 72 characters, body explains why.
- A change that moves a boundary or forecloses an option gets an ADR in `docs/decisions/`.

## Skills

`.claude/skills/README.md` lists them. Use `rust-skills` when writing Rust, `kernel-authoring` for
compute backends and GPU kernels, `test-driven-development` for features and fixes,
`systematic-debugging` for bugs, `verification-before-completion` before claiming anything works,
`security-audit-standard` before a release, `/code-quality` for cleanup, `/adr` to record a
decision. `/check` is the gate.

## Out of scope

See the same section in `docs/ROADMAP.md`. Don't build toward those without an explicit decision.
