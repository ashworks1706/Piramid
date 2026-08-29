# CPU image. Built and pushed by .github/workflows/cd.yml.
#
# Uses cargo-chef so dependency compilation is cached in its own layer: an eleven-crate workspace
# otherwise rebuilds every dependency on any source change, which dominates CI time.

FROM rust:1.87-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# Record just the dependency graph, so the next stage's cache key ignores source edits.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Cached unless a manifest changes.
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --bin piramid

FROM debian:bookworm-slim AS runtime

LABEL org.opencontainers.image.title="Piramid" \
      org.opencontainers.image.description="Inference engine for retrieval systems." \
      org.opencontainers.image.source="https://github.com/ashworks1706/piramid" \
      org.opencontainers.image.url="https://piramiddb.com" \
      org.opencontainers.image.licenses="Apache-2.0"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Unprivileged: the server needs no root capability, and the data volume is chowned to it.
RUN useradd --system --create-home --uid 10001 piramid
WORKDIR /app
COPY --from=builder /app/target/release/piramid /usr/local/bin/piramid
RUN mkdir -p /data && chown piramid:piramid /data
USER piramid

ENV PORT=6333 \
    DATA_DIR=/data \
    RUST_LOG=info

VOLUME ["/data"]
EXPOSE 6333

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -fsS http://localhost:6333/api/health || exit 1

ENTRYPOINT ["piramid"]
CMD ["serve"]
