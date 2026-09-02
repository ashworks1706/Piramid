---
name: check
description: Run the full gate (`just check`: fmt, clippy, tests, dependency rule, website) and fix whatever it reports. Use before every commit and after finishing any code change.
---

# check

Run `just check` from the repo root.

For a single area the narrower recipes are fine:

- `just check-rust` — `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace`, `./scripts/check-deps.sh`
- `just check-website` — eslint
- `just check-features` — compile-checks `gpu-cuda`, `inference-candle`, `otel`, and `--all-features`

CI and the pre-commit hook run these same recipes, so local green means CI green.

## Fixing failures

Fix at the source. Never add an `#[allow(...)]`, delete an assertion, or skip a test to go green.
If a lint is genuinely wrong for one case, the allow goes on the smallest possible scope with a
one-line comment saying why.

What the common failures actually mean:

**`check-deps.sh` fails.** You added a dependency edge that breaks the layering in
`docs/ARCHITECTURE.md`. That's a design problem, not a script problem. Either the code belongs in
a different crate, or the boundary genuinely needs to move, in which case update
`scripts/check-deps.sh` and `docs/ARCHITECTURE.md` in the same commit and say why in the message.

**`panic`, `unwrap`, or `print_stdout` denied.** These are denied outside `apps/cli` on purpose. In
a library, return a `Result`. In a test the allow is already configured in `clippy.toml`; if clippy
still complains, your helper isn't marked `#[test]` and needs a narrowly scoped allow.

**`unsafe_code` denied.** Four sites allow it, all documented in SECURITY.md; a fifth fails the
security workflow. Needing `unsafe` is a design conversation first.

**MSRV failure.** You used a std API newer than `rust-version` in the root `Cargo.toml`. Either
avoid it or bump the MSRV in `Cargo.toml` and `clippy.toml` together.

## Reporting

Say which step failed, what you changed, and the final status. If you couldn't fix something, say
so plainly rather than working around it.
