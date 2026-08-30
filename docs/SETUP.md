# Setup

Local development on Linux, macOS, and Windows through WSL2. On Windows use WSL2 with a Linux
distribution and run the commands below; PowerShell isn't covered.

For running published images see [`deploy/README.md`](../deploy/README.md). For CI and release
workflows see [`.github/workflows/`](../.github/workflows/).

## Prerequisites

| Tool | Needed for | Where |
|---|---|---|
| Rust 1.87+ | everything | https://rustup.rs |
| `just` | every task | https://just.systems |
| `jq` | `scripts/check-deps.sh` | your package manager |
| Docker | `just up` | https://docs.docker.com/engine/install |
| Node 20+ | the website | https://nodejs.org |
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
just doctor       # checks every tool above
```

`just doctor` prints ok, warn, or miss per tool and exits non-zero if a required one is missing.

## Run

```bash
just serve                    # server on http://0.0.0.0:6333
just cli show config          # print the resolved configuration
just cli show metrics         # print metrics without starting the server
just cli support-bundle       # diagnostics for a bug report
```

Check it's up:

```bash
curl -s http://localhost:6333/api/health
```

## The gate

```bash
just check          # everything
just check-rust     # fmt, clippy, tests, layering only
just fmt            # format in place
```

The pre-commit hook, installed by `just hooks`, runs the gate for whichever units your staged
changes touch. `git commit --no-verify` skips it once.

## Configuration

Settings resolve in this order, with later winning:

1. defaults in `apps/engine/core/src/config`
2. a YAML or JSON file named by `CONFIG_FILE`
3. environment variables

Every variable is documented in [`.env.example`](../.env.example), and `just env` copies it to
`.env`. Both compose files read `.env` automatically.

Invalid configuration fails at startup with a message naming the variable, not at runtime.

## Feature builds

```bash
just check-gpu          # compile-check --features gpu-cuda, no GPU needed
just check-inference    # compile-check --features inference-candle
just check-features     # both, plus --all-features
```

Features are additive and off by default. `EXECUTION_MODE=gpu` on a build without `gpu-cuda` is
rejected at startup. On a build with it but no device present, dispatch logs a warning and falls
back to CPU.

## Docs and benchmarks

```bash
just doc          # rustdoc, warnings are errors
just doc-open     # and open it
just bench        # criterion, results in target/criterion
just audit        # cargo-deny: advisories, bans, licences, sources
```

## Website

Next.js 16 on React 19, TypeScript, Tailwind 4, and MDX for the blog. It is not part of the
workspace build and ships with nothing.

```bash
just web-setup      # npm ci, once
just web            # dev server with hot reload on :3000
just web-build      # production build
just web-preview    # build and serve what actually deploys
just web-shots      # headless screenshots into target/screenshots
just check-website  # eslint
just web-frames     # regenerate the landing animation from the CLI's frames
```

`just web` runs a dev bundle, which hides prerender and font-loading problems. Check `web-preview`
before pushing.

`web-shots` needs `google-chrome`. Reading the markup is not enough to review a design; it has
already caught a stylesheet that never loaded and an animation frozen on its first frame.

## Troubleshooting

**`just: command not found`** — install from https://just.systems, or run the underlying cargo
commands directly. `just --list` shows what each recipe does.

**`check-deps: jq is required`** — install `jq`.

**Disk fills during builds** — the workspace `target/` directory grows quickly. `just clean`
removes it along with `node_modules`.

**Port 6333 already in use** — `PORT=7333 just serve`.

**Where test data goes** — `target/tmp/`, via `CARGO_TARGET_TMPDIR`. Safe to delete.
