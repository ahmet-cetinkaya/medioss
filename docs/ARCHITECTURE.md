# Architecture Specification

## System Overview

The MediOSS wellness application follows a **Clean Architecture** pattern organized as a Rust workspace with an Astro frontend, designed for performance, safety, and single-binary deployment simplicity.

```
┌─────────────────────────────────────────────────────────────────────┐
│                       PRESENTATION LAYER                            │
│                                                                     │
│  ┌────────────────────────────┐  ┌────────────────────────────┐     │
│  │  web-api (Rust/Axum)       │  │  web-ui (Astro/TypeScript) │     │
│  │  Routes, Controllers, DTOs │  │  Pages, Components, Islands│     │
│  └────────────────────────────┘  └────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
                              │                    │
                              ▼                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       INFRASTRUCTURE LAYER                          │
│                                                                     │
│  ┌────────────────────┐  ┌────────────────────┐                     │
│  │  persistence       │  │  file              │                     │
│  │  (Rust/SeaORM)     │  │  (Rust/storage)    │                     │
│  └────────────────────┘  └────────────────────┘                     │
└─────────────────────────────────────────────────────────────────────┘
                              │                    │
                              ▼                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          CORE LAYER                                 │
│                                                                     │
│  ┌──────────────────────────────┐  ┌────────────────────────────┐   │
│  │  application (CQRS/Mediator)  │  │  domain                   │   │
│  │  (Rust/commands & queries)   │  │  (Rust/entities & traits)  │   │
│  └──────────────────────────────┘  └────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Dependency Rule

Dependencies flow **inward only** — outer layers depend on inner layers, never the reverse.

```
presentation → application → domain
infrastructure → domain
```

The `domain` package is the innermost. It has zero dependencies on other crates, frameworks, or external services.

---

## Directory Structure

```
src/
├── core/
│   ├── domain/                    # Entities, repository traits (pure Rust)
│   │   ├── shared/                #   cross-cutting types
│   │   └── features/{feature}/    #   entity.rs, repository.rs
│   │
│   └── application/               # CQRS via mediator pattern
│       ├── shared/                #   mediator, command/query/handler traits
│       └── features/{feature}/    #   commands/, queries/
│
├── infrastructure/
│   ├── persistence/               # SeaORM, migrations, repo implementations
│   │   ├── migrations/
│   │   └── features/{feature}/    #   entity.rs, repository.rs
│   │
│   └── file/                      # Local file storage, audio metadata
│
└── presentation/
    ├── web-api/                   # Axum HTTP API (composition root)
    │   ├── media/                 #   user uploads
    │   ├── static/                #   Astro build output
    │   └── features/{feature}/    #   controller.rs, dto.rs
    │
    └── web-ui/                    # Astro/TypeScript frontend
        ├── pages/                 #   file-based routing
        ├── shared/                #   api, components, layouts, stores
        └── features/{feature}/    #   components, stores, *.astro
```

---

## Workspace Configuration

A top-level `Cargo.toml` workspace ties all Rust crates together:

```toml
[workspace]
members = [
    "src/core/application",
    "src/core/domain",
    "src/infrastructure/persistence",
    "src/infrastructure/file",
    "src/presentation/web-api",
]
resolver = "2"
```

The `domain` crate is depended on by every other crate. The `application` crate is depended on only by `web-api`. Infrastructure crates are depended on only by `web-api`.

---

## Composition Root

`presentation/web-api/src/main.rs` is the composition root. It is the only place where concrete implementations are wired together:

```
┌────────────────────────────────────────────────────────────────┐
│  web-api/main.rs                                               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  1. Initialize config (env vars, settings)               │  │
│  │  2. Open SeaORM DatabaseConnection                       │  │
│  │  3. Run migrations                                       │  │
│  │  4. Create persistence repo implementations              │  │
│  │  5. Create file storage implementation                   │  │
│  │  6. Register command & query handlers with mediator      │  │
│  │  7. Create controllers, injecting mediator               │  │
│  │  8. Build Axum Router, register routes & middleware      │  │
│  │  9. Serve static files & media                           │  │
│  │ 10. Start server                                         │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

This is the only place that knows about concrete types. Every other crate depends only on the abstractions declared in `core/domain`.

---

## Security Architecture

```
┌──────────────────────────────────────────────┐
│          Reverse Proxy (Caddy/Nginx)         │
│          TLS termination, rate limiting      │
└──────────────────────────────────────────────┘
                       │
┌──────────────────────────────────────────────┐
│        API Layer - Input Validation          │
│        DTO validation, file type checks      │
└──────────────────────────────────────────────┘
                       │
┌──────────────────────────────────────────────┐
│     Application Layer - Authorization        │
│     Use case permissions, rate limits        │
└──────────────────────────────────────────────┘
                       │
┌──────────────────────────────────────────────┐
│    Infrastructure Layer - Security Controls  │
│    SQL injection prevention (SeaORM),        │
│    File path traversal protection,           │
│    File size limits                          │
└──────────────────────────────────────────────┘
```
