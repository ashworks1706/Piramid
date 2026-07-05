# DevOps

This document explains how Piramid is checked, packaged, and released.

Keep this separate from `setup.md`. Setup is for getting a local machine ready. DevOps is for understanding the automation around CI, releases, crates.io, Docker images, and GitHub release artifacts.

## Local Checks

The main local check script is:

```bash
./scripts/check.sh
```

It runs:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo nextest run --all-targets`
- `cargo build --all-targets`

The website check script is:

```bash
./scripts/check-website.sh
```

If local hooks are enabled, the pre-push hook runs both checks:

```bash
git config core.hooksPath .githooks
```

This is meant to catch formatting, lint, Rust tests, and website issues before CI has to do it.

## CI Workflows

### Rust

The Rust workflow runs on pushes and pull requests to `main`.

It installs stable Rust with `rustfmt` and `clippy`, installs `cargo-nextest`, then runs the same Rust check script used locally.

### Security

The security workflow runs on pushes and pull requests to `main`, plus a weekly schedule.

It runs `cargo deny check advisories bans licenses sources` against `Cargo.lock` with repository policy from `deny.toml`. Dependency review only runs on pull requests because GitHub's dependency review action is PR-focused.

Security warnings are not all equal. A blocking vulnerability should stop a release. A known warning can be accepted temporarily, but it should be tracked and cleaned up intentionally.

### Release-plz

The `release-plz` workflow is enabled as a scaffold:

- Pushes to `main` open/update a release PR (`release-plz release-pr`).
- Publishing (`release-plz release`) is manual-only through workflow dispatch with `publish=true`.

This keeps automated version/changelog PRs without forcing an immediate migration away from the existing `releases` branch release pipeline.

### Docker

The Docker workflow runs on pull requests to `main`.

It builds the Docker image for validation only. It does not push an image. Docker publishing is release-owned so normal pushes do not create public images accidentally.

### Website

The website workflow only runs when files under `website/` or the website workflow itself change.

This keeps Rust-only changes from paying the website CI cost.

## Release Flow

The release workflow runs from the `releases` branch or by manual dispatch.

The intended release path is:

```text
prepare
  -> build-binaries
  -> publish-crate
  -> publish-docker
  -> create-release
```

The important rule is that publishing waits for binary builds. If a platform build fails, crates.io, GHCR, and the final GitHub release should not be published.

## Prepare

The prepare job:

- resolves the version from `Cargo.toml` or the manual workflow input
- validates the semver value
- verifies that the tag does not already exist
- runs `./scripts/check.sh`
- runs `cargo package --locked`
- creates and pushes the release tag

The tag uses the `v` prefix. Version `0.2.0` becomes tag `v0.2.0`.

If this job creates the tag and a later job fails, delete the partial tag before retrying the same version:

```bash
gh release delete v0.2.0 --cleanup-tag -y
git tag -d v0.2.0 2>/dev/null || true
```

If the crate has already been published to crates.io, do not retry the same version. Bump to the next patch version.

## Binary Builds

The release workflow builds platform binaries for:

- Linux amd64
- Linux arm64
- macOS amd64
- macOS arm64
- Windows amd64

Linux arm64 is cross-compiled from Ubuntu. The workflow installs the ARM64 linker and uses the target-specific linker environment variable.

The binaries are uploaded as workflow artifacts first. The GitHub release is created later after publishing succeeds.

Each binary artifact is also attested with GitHub Artifact Attestations (`actions/attest-build-provenance`) so you can verify provenance from workflow run to release asset.

## crates.io Publishing

The crate is published with OIDC Trusted Publishing:

```bash
cargo publish --locked
```

The workflow exchanges an OIDC token using `rust-lang/crates-io-auth-action@v1` and does not require a long-lived `CARGO_REGISTRY_TOKEN` secret.

If the `crates-io` environment is enabled in GitHub, make sure the job has `id-token: write` permission and that your crate is configured for trusted publishing in crates.io settings.

## Docker Publishing

Release Docker images are pushed to GHCR, not from the normal Docker workflow.

The release workflow publishes:

- `ghcr.io/ashworks1706/piramid:vX.Y.Z`
- `ghcr.io/ashworks1706/piramid:X.Y.Z`
- `ghcr.io/ashworks1706/piramid:latest`
- `ghcr.io/ashworks1706/piramid:sha-*`

The image includes OCI metadata for title, description, source, URL, and license.

The release workflow also generates an SBOM for the pushed image and uploads it as a workflow artifact.
It also attests pushed image provenance with `actions/attest-build-provenance`.

## GitHub Release

The GitHub release is created last.

Release notes are read from the first matching file:

```text
release-notes/vX.Y.Z.md
release-notes/X.Y.Z.md
release-notes/latest.md
```

The workflow also enables GitHub-generated release notes. The written release note file gives the release a human summary, and GitHub adds commit/PR-derived details.

## Required GitHub Settings

The repository needs Actions write permissions so the workflow can push tags and publish release assets:

```text
Settings -> Actions -> General -> Workflow permissions -> Read and write permissions
```

GHCR publishing uses `GITHUB_TOKEN`, so it usually does not need a separate token.

Trusted publishing also requires crates.io configuration per crate:

```text
crates.io -> Crate Settings -> Trusted Publishing
```

Set this repository/workflow as a trusted publisher before removing legacy API-token publishing.

## Why Checks Show On Multiple Branches

GitHub attaches check results to commit SHAs.

If `main` and `releases` point at the same commit, release checks can appear when viewing either branch. That does not mean the same workflow ran twice. It means GitHub is showing all checks attached to that commit.

## Release Commands

Typical release:

```bash
git checkout main
git pull
git checkout -B releases
git push origin releases
```

If release fixes are committed directly on `releases`, push that branch after committing:

```bash
git add .github/workflows/release.yml Cargo.toml Cargo.lock Dockerfile
git commit -m "fix release pipeline"
git push origin releases
```

For a new version, update `Cargo.toml`, refresh `Cargo.lock`, write a release note, and then push `releases`.
