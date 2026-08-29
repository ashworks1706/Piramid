# Piramid tasks. `just` lists them; `just <recipe>`.
# Crates: core, compute, gpu, storage, index, search, collections, embeddings,
# inference, server (crates/) · piramid CLI (apps/cli) · website (TypeScript)

set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list --unsorted

# ---------- first run ----------

# Check required tools, .env, and git hooks
doctor:
    ./scripts/doctor.sh

# Create .env from the example (no-op if it exists)
env:
    @[ -f .env ] && echo ".env exists" || { cp .env.example .env && echo "created .env"; }

# Install the pre-commit hook (runs the gate for touched crates)
hooks:
    git config core.hooksPath .githooks
    @echo "hooks installed: .githooks/pre-commit"

# Everything a fresh clone needs
bootstrap: env hooks setup
    @echo "ready: 'just serve' to run, 'just check' before you commit"

# Fetch dependencies for every unit
setup:
    cargo fetch
    cd website && npm ci

# ---------- the gate ----------

# Format, lint, test, and verify layering. CI and the pre-commit hook run this.
check: check-rust check-website
    @echo "all units ok"

# fmt --check, clippy -D warnings, tests, dependency direction
check-rust:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    ./scripts/check-deps.sh

check-website:
    cd website && npm run lint

# Format every unit in place
fmt:
    cargo fmt --all
    cd website && npx eslint . --fix

# ---------- feature matrices ----------

# Compile-check the GPU backend without a GPU present
check-gpu:
    cargo check --workspace --features gpu-cuda --all-targets

# Compile-check the inference backend
check-inference:
    cargo check --workspace --features inference-candle --all-targets

# Every feature combination CI builds
check-features: check-gpu check-inference
    cargo check --workspace --all-features --all-targets

# ---------- run ----------

# Run the server (defaults to 0.0.0.0:6333)
serve *ARGS:
    cargo run -p piramid -- serve {{ARGS}}

# Any CLI subcommand: just cli show config
cli *ARGS:
    cargo run -p piramid -- {{ARGS}}

# ---------- docs, benches, security ----------

# Build rustdoc for the workspace; warnings are errors
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features

# Open the docs
doc-open: doc
    cargo doc --workspace --no-deps --open

# Criterion benchmarks
bench *ARGS:
    cargo bench --workspace {{ARGS}}

# Advisories, licences, bans, sources
audit:
    cargo deny check advisories bans licenses sources

# ---------- containers ----------

up *ARGS:
    docker compose -f deploy/compose.yml up -d {{ARGS}}

down:
    docker compose -f deploy/compose.yml down

logs *ARGS:
    docker compose -f deploy/compose.yml logs -f {{ARGS}}

# Production images from GHCR (PIRAMID_IMAGE_TAG=main|<sha>)
prod-up *ARGS:
    docker compose -f deploy/compose.yml -f deploy/compose.prod.yml pull
    docker compose -f deploy/compose.yml -f deploy/compose.prod.yml up -d {{ARGS}}

prod-down:
    docker compose -f deploy/compose.yml -f deploy/compose.prod.yml down

# Build images locally
images:
    docker build -f deploy/docker/piramid.Dockerfile -t piramid .

# ---------- cleanup ----------

clean:
    cargo clean
    rm -rf website/node_modules website/.next
