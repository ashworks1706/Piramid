---
name: check
description: Run the full gate (`just check`: fmt, clippy, tests, dependency law, website) and fix whatever it reports. Use before every commit and after finishing any code change.
---

# check

Run `just check` from the repo root.

For a single area, the narrower recipes are fine:

- `just check-rust` — `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace`, `./scripts/check-deps.sh`
- `just check-website` — eslint
- `just check-features` — compile-checks `gpu-cuda`, `inference-candle`, and `--all-features`

CI and the pre-commit hook run these same recipes, so local green means CI green.

## Fixing failures

Fix at the source. Never `#[allow(...)]` a lint, never delete an assertion, never skip a test to
go green. If a lint is genuinely wrong for one case, the allow goes on the smallest possible scope
with a one-line comment saying why.

Common failures and what they actually mean:

**`check-deps.sh` fails** — you added a dependency edge that violates the layering in
`docs/ARCHITECTURE.md`. This is a design problem, not a script problem. Either the code belongs in
a different crate, or the boundary genuinely needs to move — in which case update
`scripts/check-deps.sh` *and* `docs/ARCHITECTURE.md` in the same commit, and say why in the message.

**`panic`/`unwrap`/`print_stdout` denied** — these are denied outside `apps/cli` on purpose. In a
library, return a `Result`. In a test, the allow is already configured in `clippy.toml`; if clippy
still complains, your helper is not marked `#[test]` and needs a narrowly scoped allow.

**`unsafe_code` denied** — `crates/gpu` allows it. Everywhere else, three sites are documented in
SECURITY.md and a fourth fails the security workflow. If you need `unsafe`, that is a design
conversation first.

**MSRV failure** — you used a std API newer than `rust-version` in the root `Cargo.toml`. Either
avoid it or bump the MSRV in `Cargo.toml` and `clippy.toml` together.

## Reporting

Say which step failed, what you changed, and the final status. If you could not fix something, say
so plainly rather than working around it.
