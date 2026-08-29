# Setup

Local development setup for Linux, macOS, and Windows via WSL2. On Windows use WSL2 with a Linux
distribution and run the commands below; PowerShell is not covered.

For running published images, see [`deploy/README.md`](../deploy/README.md). For CI and release
workflows, see [`.github/workflows/`](../.github/workflows/).

## Prerequisites

| Tool | Needed for | Install |
|---|---|---|
| Rust ≥ 1.87 | everything | https://rustup.rs |
| `just` | every task | https://just.systems |
| `jq` | `scripts/check-deps.sh` | your package manager |
| Docker | `just up` | https://docs.docker.com/engine/install |
| Node ≥ 20 | the website | https://nodejs.org |
| CUDA toolkit | `--features gpu-cuda` only | https://developer.nvidia.com/cuda-downloads |

The default build is CPU-only and needs no CUDA toolkit.

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
```

## Clone and bootstrap

```bash
git clone https://github.com/ashworks1706/piramid
cd piramid
just bootstrap    # creates .env, installs git hooks, fetches dependencies
just doctor       # verifies every tool above
```

`just doctor` prints `ok` / `warn` / `miss` per tool and exits non-zero if a required one is
missing.

## Run

```bash
just serve                    # server on http://0.0.0.0:6333
just cli show config          # print the resolved configuration
just cli show metrics         # print metrics without starting the server
```

Verify:

```bash
curl -s http://localhost:6333/api/health
```

## The gate

```bash
just check          # everything
just check-rust     # fmt, clippy, tests, layering only
just fmt            # format in place
```

The pre-commit hook (installed by `just hooks`) runs the gate for whichever units your staged
changes touch. `git commit --no-verify` skips it once.

## Configuration

Settings resolve in this order, later winning:

1. defaults in `apps/engine/foundation/core/src/config`
2. a YAML or JSON file named by `CONFIG_FILE`
3. environment variables

Every variable is documented in [`.env.example`](../.env.example); `just env` copies it to `.env`.
Both compose files read `.env` automatically.

Invalid configuration fails at startup with a message naming the variable. It does not fail at
runtime — see [ADR 0007](decisions/0007-transport-agnostic-errors.md) for how errors are shaped.

## Feature builds

```bash
just check-gpu          # compile-check --features gpu-cuda (no GPU needed)
just check-inference    # compile-check --features inference-candle
just check-features     # both, plus --all-features
```

Features are additive and default-off. `EXECUTION_MODE=gpu` on a build without `gpu-cuda` is
rejected at startup; on a build with it but no device present, dispatch logs a warning and falls
back to CPU.

## Docs and benchmarks

```bash
just doc          # rustdoc, warnings are errors
just doc-open     # and open it
just bench        # criterion, results in target/criterion
just audit        # cargo-deny: advisories, bans, licences, sources
```

## Website

```bash
cd website && npm ci && npm run dev
```

## Troubleshooting

**`just: command not found`** — install from https://just.systems, or run the underlying `cargo`
commands directly; `just --list` shows what each recipe does.

**`check-deps: jq is required`** — install `jq`.

**Disk fills during builds** — the workspace `target/` directory grows quickly. `just clean`
removes it along with `node_modules`.

**Port 6333 already in use** — `PORT=7333 just serve`.

**Tests write to `.piramid/tests`** — that directory is gitignored and safe to delete.
