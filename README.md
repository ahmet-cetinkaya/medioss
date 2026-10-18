<p align="center">
  <img src="docs/orb.svg" alt="" width="68" align="center">
</p>

# MediOSS

A self-hosted wellness app for calmer days: a meditation library, an ambient soundscape mixer and a mood tracker, behind one account on your own server. Your data stays in one SQLite file that you control.

[![CI](https://github.com/ahmet-cetinkaya/medioss/actions/workflows/ci.yml/badge.svg)](https://github.com/ahmet-cetinkaya/medioss/actions/workflows/ci.yml) [![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)

<p align="center">
  <img src="docs/screenshots/home-desktop.png" alt="MediOSS home screen on desktop" width="720">
</p>

## Features

- **Meditation library**: upload your own guided sessions, group them in programs with covers, and track your practice
- **Soundscape mixer**: mix built-in and uploaded ambient sounds and save your mixes
- **Mood tracking**: daily mood entries with notes, history and statistics
- **Multi-user**: admin-created accounts with per-user data isolation (no public signup)
- **Data portability**: export and import your mood and practice data as JSON or CSV
- **Private by design**: no third-party services, no tracking, no accounts elsewhere
- **Responsive UI**: mobile-first "Soft Glass" design

More screens: [library](docs/screenshots/library-desktop.png), [program](docs/screenshots/program-desktop.png), [player](docs/screenshots/player-desktop.png), [soundscape](docs/screenshots/soundscape-desktop.png), [mood](docs/screenshots/mood-desktop.png), [statistics](docs/screenshots/stats-desktop.png).

## Quick start (Docker)

```bash
git clone https://github.com/ahmet-cetinkaya/medioss.git
cd medioss
cp .env.example .env     # then set SESSION_SECRET: openssl rand -base64 32
make docker-up           # build the image and start it in the background
```

Open `http://localhost:3000` and register the first admin. The first-run setup asks for your `SESSION_SECRET` as proof that you control the host. `.env` ships with `SESSION_COOKIE_SECURE=true`, which needs HTTPS; for a plain-HTTP localhost trial, set it to `false` in `.env`. `make docker-logs` follows the logs and `make docker-down` stops the stack. Details: [Docker guide](docs/docker.md) and [Deployment guide](docs/DEPLOYMENT.md).

## Quick start (development)

You need Rust (version pinned in `rust-toolchain.toml`), [bun](https://bun.sh) and `ffmpeg` on your PATH.

```bash
make run     # build the UI, then serve API + UI from one process on :3000 (data in ./.data)
make dev     # Astro dev server on :4321 with hot reload, plus the API on :3000
make lint    # prettier, cargo fmt, clippy, boundary check
make test    # bun test, cargo test --workspace, boundary-check self-test
make demo    # seed a throwaway instance and capture screenshots (see docs/demo.md)
```

Run `make` targets rather than raw `cargo` commands: they wire up the frontend build and environment. Running `cargo run -p web-api` directly exits because `MEDIOSS_DATA_DIR` has no default.

## Configuration

Settings come from environment variables; [`.env.example`](.env.example) is the full contract.

| Variable | Required | Default | Purpose |
| --- | --- | --- | --- |
| `MEDIOSS_DATA_DIR` | yes | none | Directory for the SQLite database and uploaded media |
| `SESSION_SECRET` | yes | none | Proof of host control for first-run setup; it does not sign cookies |
| `MEDIOSS_ORIGIN` | yes | none | The exact URL users open; other origins are rejected for state-changing requests |
| `PORT` | no | `3000` | Listen port |
| `MEDIOSS_BIND` | no | `0.0.0.0` | Listen address; use `127.0.0.1` to keep the server local |
| `SESSION_TIMEOUT` | no | `7d` | Session lifetime |
| `SESSION_COOKIE_SECURE` | no | `true` | Secure cookies need HTTPS; set `false` only for plain-HTTP local use |
| `MEDIOSS_DEV_ORIGIN` | no | none | Development only: one extra allowed origin, e.g. the Astro dev server (`make dev` sets it) |
| `MEDIOSS_STATIC_DIR` | no | `src/apps/web-api/static` | Built UI served by the binary |
| `MEDIOSS_BUILTIN_SOUNDS_DIR` | no | probed | Built-in sound set; falls back to `/srv/medioss/builtin-sounds`, then `./builtin-sounds` |

The binary also works as an operator CLI: `medioss reset-password <email>` and `medioss clear-throttle <identifier>` read `MEDIOSS_DATA_DIR` and need no running server.

## Tech stack

Rust with Axum and SeaORM on SQLite, shipped as one binary that serves the API and the UI. The frontend is Astro with Svelte 5 islands, Tailwind CSS and shadcn-svelte, with audio through the Web Audio API. The backend is a modular monolith with enforced layer boundaries.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) and [Design system](docs/DESIGN_SYSTEM.md)
- [Product requirements](docs/PRD.md) and [Roadmap](docs/ROADMAP.md)
- [Deployment](docs/DEPLOYMENT.md), [Docker](docs/docker.md) and [Demo data](docs/demo.md)
- [Documentation index](docs/README.md)

## Contributing and security

Contributions are welcome: see [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md). Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

## License

MediOSS is licensed under [AGPL-3.0-or-later](LICENSE). The built-in ambient sounds in `assets/sounds/soundscape-builtin/` are third-party works and are **not** covered by it; see [NOTICE](NOTICE) for these and other bundled components.
