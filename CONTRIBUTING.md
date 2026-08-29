# Contributing

## Setup

```bash
git clone https://github.com/ashworks1706/piramid && cd piramid
just bootstrap   # .env, git hooks, dependencies
just doctor      # verify tooling
```

You need Rust (stable, ≥ 1.87), [`just`](https://just.systems), and `jq`. Docker, Node, and a CUDA
toolkit are optional — the default build is CPU-only and needs no vendor toolchain.

## The gate

```bash
just check
```

Runs `cargo fmt --check`, `clippy -D warnings`, the test suite, `scripts/check-deps.sh`, and the
website lint. CI and the pre-commit hook run the same recipes, so local green means CI green.

A change is not done until it passes. Fix failures at the source — never `#[allow]` a lint or skip
a test to go green. A genuine exception gets the narrowest possible scope and a one-line reason.

## Before you write code

Read [AGENTS.md](AGENTS.md) for the layout and rules, and
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the crate boundaries and the three seams.

The one thing worth internalizing: **a crate may depend on one below it in the dependency law, and
never the reverse.** `scripts/check-deps.sh` enforces this. If your change needs a new edge, that
is a design conversation — open an issue first.

## Where code goes

| If it is… | It belongs in |
|---|---|
| HTTP-specific | `apps/engine/service/server/src/http` |
| A user-facing operation | `apps/engine/service/server/src/services` |
| One collection's state | `apps/engine/data/collections` |
| Bytes, mmap, WAL, sidecars | `apps/engine/data/storage` |
| An ANN implementation detail | `apps/engine/retrieval/index` |
| Distance math or backend dispatch | `apps/engine/hardware/compute` |
| Device memory, streams, kernels | `apps/engine/hardware/gpu` |
| Model execution | `apps/engine/inference/runtime` |
| Shared vocabulary | `apps/engine/core` |

## Adding a compute backend

One file in `apps/engine/hardware/compute/src/backends/` implementing `DistanceKernels`, one arm in the registry
in `backends/mod.rs`. Nothing else changes — that is the point of the trait.

The batch methods take a contiguous row-major slab and a caller-owned `out`. Do not change that
shape to `&[Vec<f32>]`; scattered rows cannot be uploaded to a device without a per-call gather
that costs more than the kernel saves.

New backends need a parity test against `ScalarBackend` and a bench against it.

## Good first issues

`docs/ROADMAP.md` has a **Known gaps** section. Wiring the existing PQ implementation into the
search path, and backfilling doc comments so `missing_docs` can move from `allow` to `deny`, are
both self-contained.

## Commits and PRs

- Imperative subject ≤ 72 chars. The body explains *why*, not what the diff already shows.
- One logical change per PR.
- New behavior comes with a test.
- A change that moves a boundary or forecloses an option gets an ADR in `docs/decisions/`.

## Reporting bugs

Run `piramid support-bundle` and attach the file. It collects version, platform, build features,
resolved configuration, and collection state in one pass, with credential-shaped values redacted —
read it before sharing.

Add the smallest reproduction you can manage. For a search-correctness bug, the collection size
and `k` matter.
