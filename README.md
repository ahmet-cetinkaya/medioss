<!-- markdownlint-disable-file MD041 -->

<p align="center">
  <img src="docs/orb.svg" alt="MediOSS orb" width="68">
</p>

# MediOSS

[![CI](https://img.shields.io/github/actions/workflow/status/ahmet-cetinkaya/medioss/ci.yml?logo=github&style=flat)](https://github.com/ahmet-cetinkaya/medioss/actions/workflows/ci.yml) [![GitHub stars](https://img.shields.io/github/stars/ahmet-cetinkaya/medioss?style=flat)](https://github.com/ahmet-cetinkaya/medioss/stargazers) [![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE) [![Donate](https://img.shields.io/badge/Donate-ahmetcetinkaya.me%2Fdonate-blue?style=flat)](https://ahmetcetinkaya.me/donate)

A self-hosted wellness app for calmer days: a meditation library, an ambient soundscape mixer, and a mood tracker, all behind one account on your own server. Your data stays in one SQLite file that you control.

<p align="center">
  <img src="docs/screenshots/home-desktop.png" alt="MediOSS home screen on desktop" width="720">
</p>

## ✨ Features

- **Meditation library** — upload your own guided sessions, group them in programs with covers, and track your practice
- **Soundscape mixer** — mix built-in and uploaded ambient sounds and save your mixes
- **Mood tracking** — daily mood entries with notes, history, and statistics
- **Multi-user** — admin-created accounts with per-user data isolation (no public signup)
- **Data portability** — export and import your mood and practice data as JSON or CSV
- **Private by design** — no third-party services, no tracking, no accounts elsewhere
- **Responsive UI** — mobile-first "Soft Glass" design

More screens: [library](docs/screenshots/library-desktop.png), [program](docs/screenshots/program-desktop.png), [player](docs/screenshots/player-desktop.png), [soundscape](docs/screenshots/soundscape-desktop.png), [mood](docs/screenshots/mood-desktop.png), [statistics](docs/screenshots/stats-desktop.png).

## ⏬ Installation

### 🐳 Docker (recommended)

```bash
git clone https://github.com/ahmet-cetinkaya/medioss.git
cd medioss
cp .env.example .env
# Edit .env: set SESSION_SECRET (generate with: openssl rand -base64 32)
# and MEDIOSS_ORIGIN (e.g. http://localhost:3000 or your domain)
docker compose up
```

Open `http://localhost:3000` (or your `MEDIOSS_ORIGIN`) and register the first admin.

### 📦 From source (development)

Requires: Rust + Cargo, Node.js + Bun, FFmpeg.

```bash
git clone https://github.com/ahmet-cetinkaya/medioss.git
cd medioss
cp .env.example .env
# Edit .env as above
make run  # build the UI, then serve API + UI from one process on :3000
```

## ⚙️ Configuration

Set these in `.env` or as environment variables (env vars take priority):

| Variable | Required | Default | Notes |
| --- | --- | --- | --- |
| `MEDIOSS_DATA_DIR` | ✓ | (none) | Directory for SQLite DB + media files; created if missing |
| `SESSION_SECRET` | ✓ | (none) | Strong random secret; generate: `openssl rand -base64 32` |
| `MEDIOSS_ORIGIN` | ✓ | (none) | Single origin the app is served from (scheme + host + optional port, no path) |
| `PORT` |  | 3000 | Listen port |
| `MEDIOSS_BIND` |  | 0.0.0.0 | Listen address; use `127.0.0.1` for localhost only |
| `SESSION_TIMEOUT` |  | 7d | Session lifetime (e.g. `7d`, `24h`, `30m`) |
| `SESSION_COOKIE_SECURE` |  | true | Set to `false` only for local HTTP testing (not production) |
| `MEDIOSS_BUILTIN_SOUNDS_DIR` |  | `assets/sounds/soundscape-builtin` | Built-in soundscape directory |

See [`.env.example`](.env.example) for all options and full descriptions.

## 🔧 Development

### Quick start

```bash
make dev      # run in development mode (file watchers + live reload)
make build    # build the release binary
make lint     # run linters (cargo clippy + markdownlint)
make test     # run all tests
```

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for Docker production setup and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for codebase structure.

### Stack

- **Backend** — Rust (Axum, SQLx, Tokio)
- **Frontend** — Svelte, TypeScript, Tailwind CSS
- **Storage** — SQLite (single file, no server needed)
- **Media** — FFmpeg for audio transcoding; WebP normalization for images

## 📚 Documentation

- [**ARCHITECTURE.md**](docs/ARCHITECTURE.md) — module structure, design patterns, data layout
- [**DEPLOYMENT.md**](docs/DEPLOYMENT.md) — production setup, reverse proxies, backup
- [**docker.md**](docs/docker.md) — image building and publishing
- [**DESIGN_SYSTEM.md**](docs/DESIGN_SYSTEM.md) — UI component library and design tokens
- [**PRD.md**](docs/PRD.md) — product requirements and feature details
- [**ROADMAP.md**](docs/ROADMAP.md) — future plans
- [**Security Policy**](SECURITY.md) — vulnerability reporting and supported versions

## 🤝 Contributing

Contributions are welcome. Before you start, please read [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Environment setup
- Code style and patterns
- Pull request process
- Issue reporting and testing requirements

## 📄 License

MediOSS is licensed under the [GNU Affero General Public License v3.0 or later](LICENSE) — see the license file for details.

Your instance's source code and modifications must remain available to your users under the same license.
