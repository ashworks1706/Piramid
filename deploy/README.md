# Deploy

| File | Purpose |
|---|---|
| `docker/piramid.Dockerfile` | CPU image. cargo-chef caches dependency builds in their own layer. |
| `docker/piramid-cuda.Dockerfile` | CUDA image, built with `--features gpu-cuda`. Needs `--gpus all`. |
| `compose.yml` | Dev stack, builds from source. |
| `compose.prod.yml` | Overlay that swaps in GHCR images. |

Commands here are plain `docker compose`, since deploying does not assume a repo checkout or any
of the contributor tooling. From inside a checkout, `just up`, `just down`, `just logs`,
`just prod-up`, and `just prod-down` are shorthands for the same things.

## Running a published image

Nothing to check out:

```bash
docker run -p 6333:6333 -v piramid-data:/data ghcr.io/ashworks1706/piramid:main
```

Images are published by `.github/workflows/cd.yml` on every push to `main`, tagged with both the
commit SHA and `main`. Pin to a SHA rather than `main` for anything you care about.

## Compose

From a checkout, building from source:

```bash
docker compose -f deploy/compose.yml up -d
docker compose -f deploy/compose.yml logs -f
docker compose -f deploy/compose.yml --profile ollama up -d    # add local embeddings
docker compose -f deploy/compose.yml down
```

With published images instead of a local build:

```bash
PIRAMID_IMAGE_TAG=main docker compose \
  -f deploy/compose.yml -f deploy/compose.prod.yml pull
PIRAMID_IMAGE_TAG=main docker compose \
  -f deploy/compose.yml -f deploy/compose.prod.yml up -d
```

## Configuration

Both compose files read `../.env` if it exists. See `.env.example` for every variable. Secrets like
`OPENAI_API_KEY` belong in `.env`, never in a compose file or an image.

## Notes

The server runs as uid 10001 rather than root, and data lives on the `piramid-data` volume at
`/data`. Health is `GET /api/health`, readiness is `GET /api/readyz`, and `GET /metrics` is the
Prometheus endpoint.

The CUDA image sets `EXECUTION_MODE=gpu`. With no device present, dispatch logs a warning and
falls back to CPU rather than failing requests.
