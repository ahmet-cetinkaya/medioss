# Running MediOSS with Docker

The `Dockerfile` builds the UI with bun, the binary with Rust in release mode, and ships both in a slim Debian image with `ffmpeg`. The process runs as the non-root user `medioss` (uid 10001) and serves the API and UI on port 3000.

## Start

`docker-compose.yml` runs the published image from `ghcr.io`. `docker-compose.dev.yml` builds the image from the local source instead.

```bash
cp .env.example .env     # then set SESSION_SECRET: openssl rand -base64 32
docker compose up -d     # pulls ghcr.io/ahmet-cetinkaya/medioss
```

To build from source (for development or an unreleased change), use `make docker-up`, which runs `docker compose -f docker-compose.dev.yml up -d --build`.

Open the address you set in `MEDIOSS_ORIGIN` (default `http://localhost:3000`) and register the first admin. Both compose files refuse to start when `SESSION_SECRET` or `MEDIOSS_ORIGIN` is empty. Compose reads both from your environment or from `.env`; never commit `.env`.

## Settings

| Variable | Purpose |
| --- | --- |
| `SESSION_SECRET` | Proof of possession for first-run setup. It does not sign cookies. |
| `MEDIOSS_ORIGIN` | The exact URL users open. State-changing requests from other origins are rejected. |
| `SESSION_COOKIE_SECURE` | Defaults to `true`, which needs HTTPS. Set `false` only for plain-HTTP local trials. |
| `SESSION_TIMEOUT` | Session lifetime, default `7d`. |
| `MEDIOSS_MAX_UPLOAD_BYTES` | Largest meditation audio upload in bytes, default `104857600` (100 MiB). |
| `MEDIOSS_MAX_SOUND_UPLOAD_BYTES` | Largest soundscape audio upload in bytes, default `104857600` (100 MiB). |
| `MEDIOSS_MAX_COVER_BYTES` | Largest program cover in bytes, default `5242880` (5 MiB). |

Each upload limit is a whole number from 1 to 10 GiB (`10737418240`). An invalid value stops the server at startup and the log names the variable. If you run a reverse proxy in front of the container, raise its body limit too (for nginx, `client_max_body_size`), or the proxy answers 413 before MediOSS sees the upload.

See [`.env.example`](../.env.example) for the full list.

## Data

All state lives in the `app_data` volume mounted at `/data` (SQLite database and uploaded media). Audio conversion needs scratch space, and the read-only root filesystem has no writable `/tmp`, so at startup the server empties and recreates `/data/tmp` and points `TMPDIR` at it (an upload of N bytes can briefly need about N bytes of free space there). It refuses to start when it cannot write to that folder, and prints the reason to stderr. Never point two containers at the same `/data` volume: the second one would empty the first one's scratch folder. Back it up before upgrading. The container root filesystem is read-only, all capabilities are dropped, and `no-new-privileges` is set.

## Prebuilt image

The published image supports `linux/amd64` and `linux/arm64`:

```bash
docker pull ghcr.io/ahmet-cetinkaya/medioss:1.1.2   # a pinned release
docker pull ghcr.io/ahmet-cetinkaya/medioss:latest  # the newest stable release
```

`docker-compose.yml` uses `latest` by default. Pin a release by setting `MEDIOSS_VERSION=1.1.2` in `.env`, then run `docker compose pull && docker compose up -d`.

## Operate

```bash
make docker-logs      # follow logs
make docker-restart   # restart
make docker-down      # stop and remove the containers, keeps the volume
docker compose exec backend medioss reset-password <email>
```

The health check calls `GET /api/health`. For HTTPS, put a reverse proxy in front and set `MEDIOSS_ORIGIN` to its public URL. More detail is in [DEPLOYMENT.md](DEPLOYMENT.md).

## Publishing the image

`.github/workflows/docker-publish.yml` builds the image for `linux/amd64` and `linux/arm64`. It runs only on tags: CI's `docker-smoke` job already builds and starts the amd64 image on every pull request and push to `main`, so branches are not built a second time here (the arm64 half is first built on the release tag). Only stable semver tags (`vX.Y.Z`, no prerelease suffix) publish `X.Y.Z`, `X.Y`, `X` and move `latest`, in one atomic push — `latest` always points at the last stable release, and prerelease tags like `v1.0.0-rc.1` never touch the registry.

The image is published to GitHub Container Registry as `ghcr.io/ahmet-cetinkaya/medioss`. The workflow logs in with the built-in `GITHUB_TOKEN`, so no secrets or variables are needed. After the first publish, set the package to public (package settings, Change visibility) and connect it to the repository so anyone can pull it.

## Releasing a new version

1. Bump `version` in `[workspace.package]` in the root `Cargo.toml` (every crate inherits it) and commit.
2. Push a stable tag: `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. CI publishes `X.Y.Z`, `X.Y`, `X` and `latest` to `ghcr.io/ahmet-cetinkaya/medioss`.

Manual fallback, when CI cannot publish:

```bash
docker build -t ghcr.io/ahmet-cetinkaya/medioss:X.Y.Z -t ghcr.io/ahmet-cetinkaya/medioss:latest .
docker login ghcr.io     # user name + a classic personal access token with write:packages
docker push ghcr.io/ahmet-cetinkaya/medioss:X.Y.Z
docker push ghcr.io/ahmet-cetinkaya/medioss:latest
```

Enter the token only at the login prompt. Never write it into a file, a script or a shell history. After the first publish, make the package public and connect it to the repository (see above).
