# Running MediOSS with Docker

The `Dockerfile` builds the UI with bun, the binary with Rust in release mode, and ships both in a slim Debian image with `ffmpeg`. The process runs as the non-root user `medioss` (uid 10001) and serves the API and UI on port 3000.

## Start

```bash
cp .env.example .env     # then set SESSION_SECRET: openssl rand -base64 32
make docker-up           # docker compose up -d --build
```

Open the address you set in `MEDIOSS_ORIGIN` (default `http://localhost:3000`) and register the first admin. `docker-compose.yml` refuses to start when `SESSION_SECRET` or `MEDIOSS_ORIGIN` is empty. Compose reads both from your environment or from `.env`; never commit `.env`.

## Settings

| Variable | Purpose |
| --- | --- |
| `SESSION_SECRET` | Proof of possession for first-run setup. It does not sign cookies. |
| `MEDIOSS_ORIGIN` | The exact URL users open. State-changing requests from other origins are rejected. |
| `SESSION_COOKIE_SECURE` | Defaults to `true`, which needs HTTPS. Set `false` only for plain-HTTP local trials. |
| `SESSION_TIMEOUT` | Session lifetime, default `7d`. |

See [`.env.example`](../.env.example) for the full list.

## Data

All state lives in the `app_data` volume mounted at `/data` (SQLite database and uploaded media). Back it up before upgrading. The container root filesystem is read-only, all capabilities are dropped, and `no-new-privileges` is set.

## Operate

```bash
make docker-logs      # follow logs
make docker-restart   # restart
make docker-down      # stop and remove the containers, keeps the volume
docker compose exec backend medioss reset-password <email>
```

The health check calls `GET /api/health`. For HTTPS, put a reverse proxy in front and set `MEDIOSS_ORIGIN` to its public URL. More detail is in [DEPLOYMENT.md](DEPLOYMENT.md).

## Publishing the image

`.github/workflows/docker-publish.yml` builds the image for `linux/amd64` and `linux/arm64`. Pull requests, pushes to `main`, and every run before `DOCKERHUB_IMAGE` is set, only build it (no login, no push), so CI stays green. `v*` tags publish `X.Y.Z`, `X.Y`, `X` and move `latest` to the released version; `latest` always points at the last release, not at every main push.

Set these in the repository settings before the first publish:

- Secrets `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` (a Docker Hub access token with write scope).
- Variable `DOCKERHUB_IMAGE`, for example `your-namespace/medioss`. Until it is set the workflow only builds and never publishes.
