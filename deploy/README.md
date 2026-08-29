# Deploy

| File | Purpose |
|---|---|
| `docker/piramid.Dockerfile` | CPU image. cargo-chef caches dependency builds in their own layer. |
| `docker/piramid-cuda.Dockerfile` | CUDA image, built with `--features gpu-cuda`. Needs `--gpus all`. |
| `compose.yml` | Dev stack, builds from source. `just up`. |
| `compose.prod.yml` | Overlay swapping in GHCR images. `just prod-up`. |

## Local

```bash
just up            # build and start
just logs          # follow
just up ollama     # add local embeddings (profile)
just down
```

## Production

Images are published to `ghcr.io/ashworks1706/piramid` by `.github/workflows/cd.yml` on every
push to `main`, tagged with both the commit SHA and `main`.

```bash
PIRAMID_IMAGE_TAG=main just prod-up
```

Pin to a SHA rather than `main` for anything you care about.

## Configuration

Both compose files read `../.env` if it exists — see `.env.example` for every variable. Secrets
(`OPENAI_API_KEY`) belong in `.env`, never in a compose file.

## Notes

- The server runs as uid 10001, not root. Data lives on the `piramid-data` volume at `/data`.
- Health is `GET /api/health`; readiness is `GET /api/readyz`.
- The CUDA image sets `EXECUTION_MODE=gpu`. With no device present, dispatch logs a warning and
  falls back to CPU rather than failing requests.
