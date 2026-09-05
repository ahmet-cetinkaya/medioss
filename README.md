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
# Clone and deploy
git clone https://github.com/ahmet-cetinkaya/medioss.git
cd medioss
docker-compose up -d
```

Visit `http://localhost:3000` to access your wellness app.

## 📋 Documentation

- [Product Requirements Document](docs/PRD.md) - Comprehensive product specification
- [Technical Architecture](docs/ARCHITECTURE.md) - Rust + Astro system design
- [Deployment Guide](docs/DEPLOYMENT.md) - Self-hosting instructions

## 🛠️ Tech Stack

- **Backend:** Rust with Axum web framework (performance & safety)
- **Database:** SQLite with SeaORM (type-safe query builder & migrations)
- **Frontend:** Astro with TypeScript
- **Styling:** Tailwind CSS
- **Audio:** Web Audio API with Howler.js
- **Deployment:** Docker & docker-compose (single binary)

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
- [x] Technical architecture designed (Rust + Astro)
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

```bash
# Backend development (Rust workspace)
cargo run -p web-api

# Frontend development (Astro)
cd src/presentation/web-ui
npm run dev

# Production build
cd src/presentation/web-ui && npm run build && cd -
cargo build --release
docker-compose up
```
