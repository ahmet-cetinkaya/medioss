# MediOSS Wellness App

A self-hosted wellness application supporting mood regulation through meditation and white noise.

## 🎯 Vision

Provide individuals with a private, self-hosted sanctuary for mental wellness through accessible meditation and ambient soundscapes.

## ✨ Features

- **🧘 Meditation Library** - Upload your own guided sessions with progress tracking
- **🎵 White Noise Player** - Customizable ambient soundscapes
- **📊 Mood Tracking** - Daily mood logs with trends and insights
- **🔐 Multi-User** - Accounts with per-user data isolation
- **🔒 Privacy First** - 100% self-hosted, no third-party dependencies
- **📱 Offline Ready** - Core features work without internet

## 🚀 Quick Start

```bash
git clone https://github.com/ahmet-cetinkaya/medioss.git
cd medioss

cp .env.example .env          # then set SESSION_SECRET: openssl rand -base64 32
make up                       # build the image and start it in the background
```

Visit `http://localhost:3000` to access your wellness app. `make logs` follows the container output and `make down` stops it.

`make up` reads `SESSION_SECRET` from your environment or `.env` and refuses to start if it is empty. MediOSS uses it only as the constant-time possession proof that authorizes first-run setup; opaque session cookies contain random server-side IDs and are not signed with this secret.

## 📋 Documentation

- [Product Requirements Document](docs/PRD.md) - Comprehensive product specification
- [Technical Architecture](docs/ARCHITECTURE.md) - Rust + Astro system design
- [Deployment Guide](docs/DEPLOYMENT.md) - Self-hosting instructions

## 🛠️ Tech Stack

- **Backend:** Rust with Axum web framework (performance & safety)
- **Database:** SQLite with SeaORM (type-safe query builder & migrations)
- **Frontend:** Astro with TypeScript, Svelte 5 islands for interactivity
- **Styling:** Tailwind CSS with shadcn-svelte components
- **Audio:** Web Audio API with Howler.js
- **Deployment:** Docker & Docker Compose (single binary)

## 🎨 Design Principles

- Calm, minimalist interface
- Progressive disclosure of features
- Mobile-first responsive design
- WCAG 2.1 AA accessibility compliance

## 🚀 Rust Backend Benefits

- **Performance:** Memory-safe with zero-cost abstractions
- **Safety:** Type-safe queries with SeaORM prevent SQL injection
- **Deployment:** Single binary, minimal dependencies
- **Efficiency:** 64-128MB RAM vs 512MB+ for Node.js
- **Concurrency:** Fearless parallel audio processing

## 📊 Current Status

🚧 **Development Status:** Early Planning Phase

- [x] Product requirements defined
- [x] Technical architecture designed (Rust + Astro/Svelte)
- [ ] Core Rust backend implementation
- [ ] Astro frontend development
- [ ] Authentication and user management
- [ ] File upload functionality

## 🤝 Contributing

Contributions welcome! This project aims to provide accessible mental wellness tools with complete user privacy.

## 📄 License

See [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built for individuals seeking private, comprehensive wellness tools without subscription costs or data harvesting.

## 🔧 Development

Every workflow goes through the `Makefile`. Use the targets rather than the underlying `cargo` and `npm` commands: they wire up the frontend build and the required environment for you.

```bash
make run     # build the UI, then serve API + UI from one process on :3000
make dev     # Astro dev server with hot reload on :4321
make test    # the same gates CI runs: fmt, clippy, cargo test, boundary check
make build   # release binary at target/release/medioss, UI bundled in
make up      # run it in Docker exactly as deployed, on :3000
make down    # stop and remove the Docker stack
```

`make run` is the one to reach for day to day. Running `cargo run -p web-api` directly exits immediately, because `MEDIOSS_DATA_DIR` has no default and the app refuses to start without it; `make run` points it at `./.data` and rebuilds the Astro bundle into the API's static directory first. Override the location with `make run DATA_DIR=/srv/medioss`.

For frontend work, run `make dev` for hot reload and `make run` in a second terminal for the API behind it. Two more targets round things out: `make logs` follows the Docker output, and `make format` rewrites Markdown with Prettier and Rust with `cargo fmt`.

See [`.env.example`](.env.example) for the full environment contract. Only `MEDIOSS_DATA_DIR` is read at startup today.
