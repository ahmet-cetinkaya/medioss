# Architecture Specification

## System Overview

MediOSS is a **modular monolith**: a single Cargo workspace of small, independent Rust crates that compile into one Axum binary, paired with one Astro/TypeScript frontend using Svelte 5 islands for interactivity. There is one process, one SQLite database, and one deployment artifact.

The workspace is split by **business module first**, then by Clean Architecture layer inside each module. A module owns its domain, its use cases, its persistence, and its HTTP surface. Nothing is shared across modules except a deliberately tiny kernel and narrowly scoped technical crates under `shared/`.

All source components live under the top-level `src/` directory. There is no top-level `crates/` directory; deployable entry points live in `src/apps/`.

```text
        web-ui (Astro/TypeScript + Svelte islands)
                  │ HTTP / static assets
        web-api (Axum binary, composition root)
                  │ mounts module routers
   ┌──────────────┼──────────────┐
┌──▼─────────┐ ┌──▼───────┐ ┌────▼─────┐   domain / application /
│ meditation │ │ <module> │ │ <module> │   infrastructure / presentation
└──┬─────────┘ └──┬───────┘ └────┬─────┘
   │              │              │
   ├──────────────┴──────────────┤  infrastructure only, when it stores blobs
   │    media-store (shared)     │
   └──────────────────────────────┘
                kernel
   (every domain/application/infrastructure/presentation crate depends on it;
    media-store does not)
```

As of v1.1.0 there are five business modules: `identity`, `meditation`, `mood-tracker`, `portability` and `soundscape`. `meditation` is used throughout as the single worked example; the same layering applies to every module. New modules follow the same pattern and are added when a feature needs a new boundary.

---

## Architectural Style

**Modular monolith with per-module Clean Architecture.**

| Property | Decision |
| --- | --- |
| Deployment | One Rust binary plus one Astro static bundle |
| Process model | Single process, single Tokio runtime |
| Database | One SQLite file, one schema, per-module migrations |
| Composition | Constructor injection at one composition root |
| Module boundary | Enforced by the Cargo dependency graph, not a runtime registry |

---

## Module Pattern

Every business module is independent Cargo crates, one per Clean Architecture layer it actually uses — a layer with no content (for example a module with no domain rules of its own) gets no crate:

| Layer | Crate name | Contents |
| --- | --- | --- |
| `domain` | `<module>-domain` | Entities, value objects, invariants, domain errors, and domain-level service interfaces only when the domain model itself needs them |
| `application` | `<module>-application` | Commands, queries, use-case services, repository and storage port traits, application errors |
| `infrastructure` | `<module>-infrastructure` | SeaORM entities, mappers, migrations, repository implementations, storage adapters |
| `presentation` | `<module>-presentation` | Axum routes, extractors, DTOs, a `router()` entry point |

Package names use `<module>-<layer>`; the corresponding Rust import path is `<module>_<layer>` (package `meditation-domain`, imported as `meditation_domain`).

**Application entry points versus module presentation.** `src/modules/<module>/presentation` is one module's Clean Architecture presentation layer: the routes and DTOs through which that module is reachable. `src/apps` contains deployable or buildable entry points: the Axum binary that composes all modules and the Astro frontend. Calling this directory `apps` avoids confusing an application host with a module's presentation adapter.

Adding a module means adding a directory under `src/modules/` with these four crates. No central registry, no shared trait to implement, no change to any existing module. The only edit outside the new directory is mounting its router in the composition root.

---

## Directory and Crate Structure

```text
medioss/
├── Cargo.toml                              # workspace root
├── docs/
└── src/
    ├── kernel/
    │   ├── Cargo.toml
    │   └── src/lib.rs
    │
    ├── modules/
    │   ├── meditation/                     # worked example
    │   │   ├── domain/
    │   │   │   ├── Cargo.toml
    │   │   │   └── src/
    │   │   │       ├── lib.rs
    │   │   │       ├── meditation.rs       # entity and its invariants
    │   │   │       ├── category.rs         # value objects: Category, Tag
    │   │   │       ├── streak.rs           # streak rules
    │   │   │       └── error.rs            # domain errors
    │   │   ├── application/
    │   │   │   ├── Cargo.toml
    │   │   │   └── src/
    │   │   │       ├── lib.rs
    │   │   │       ├── ports/              # one trait per file: MeditationRepository, ...
    │   │   │       ├── commands/           # <use-case>/ folders: command + handler + validator, each with tests
    │   │   │       ├── queries/            # <use-case>/ folders: same shape with Query, read-only
    │   │   │       ├── services/           # implementations shared by several use cases (when they exist)
    │   │   │       └── models/             # shared data contracts + models/errors/
    │   │   ├── infrastructure/
    │   │   │   ├── Cargo.toml
    │   │   │   └── src/
    │   │   │       ├── lib.rs
    │   │   │       ├── entities/           # SeaORM entities
    │   │   │       ├── mappers/            # entity <-> domain
    │   │   │       ├── migrations/         # module-owned tables and ledger
    │   │   │       ├── repository.rs       # MeditationRepository impl
    │   │   │       └── storage.rs          # MeditationAudioStore -> media-store
    │   │   └── presentation/
    │   │       ├── Cargo.toml
    │   │       └── src/
    │   │           ├── lib.rs              # module exports, pub use router::SharedDependencies
    │   │           ├── router.rs           # calls each route's register() helper
    │   │           ├── handlers/           # one file per route: request DTO + handler + register()
    │   │           ├── formats/            # serialization (json/csv), each with .tests.rs
    │   │           ├── support/            # cross-cutting helpers (origin, multipart, responses)
    │   │           └── extractors.rs       # CurrentActor and friends
    │   │
    │   └── <module>/                       # future modules follow the same shape
    │       ├── domain/
    │       ├── application/
    │       ├── infrastructure/
    │       └── presentation/
    │
    ├── shared/                              # reusable technical crates only
    │   └── media-store/
    │       ├── Cargo.toml
    │       └── src/
    │           ├── lib.rs                  # put/open/delete by opaque key
    │           ├── key.rs                  # StorageKey generation
    │           └── local_fs.rs             # $MEDIOSS_DATA_DIR/media backend
    │
    └── apps/                               # deployable or buildable entry points
        ├── web-api/
        │   ├── Cargo.toml                  # package "web-api", binary "medioss"
        │   └── src/
        │       ├── main.rs                 # composition root, thin
        │       ├── config/                 # env parsing and validation
        │       ├── bootstrap/              # db, migrations, media-store
        │       ├── state.rs                # AppState of module services
        │       ├── adapters/               # per-module port implementations (media_store_blobs, ffmpeg_probe, ...)
        │       ├── middleware.rs           # session -> Actor
        │       ├── router.rs               # mounts module routers + static
        │       └── static/                 # Astro build output (gitignored)
        │
        └── web-ui/                         # Astro/TypeScript + Svelte 5, not a Cargo member
            ├── package.json
            └── src/{pages,shared,features}/
```

---

## File Organization Inside a Crate

Layer boundaries are enforced by the Cargo graph; these rules govern the files _within_ one crate. They apply to every crate in the workspace, including each new module's four layer crates.

**Group by domain topic in domain and application.** Those crates are organized into folders named after the concern they own, not the kind of construct they hold: a topic's type, the functions operating on it, and its tests live together. There are no `types/` or `utils/` buckets, and no single `error.rs` dump — errors belong to the topic or use case that owns them, shared application errors live in `models/errors/`, and shared data contracts cross-cutting several use cases live in `models/` (`MoodRecord`, `ExportDocument` in the portability pilot). The same applies at the domain boundary: when several commands hand the same field set to one aggregate, that shared input type moves beside the aggregate as `<topic>/<topic>_input.rs` rather than being redeclared per command (`Policies/Inputs/` in the InsurUp codebase is the shape this avoids duplicating). A `models/` folder is a home for shared contracts, never a catch-all: a type belongs there only when more than one use case speaks it.

**Presentation groups by role.** The organization axis there is the job a piece of code does, not the business topic: `handlers/` (one file per route: its handler function and its `register()` helper, plus the request DTO when one exists), `formats/` (serialization), `support/` (cross-cutting helpers), plus a `router.rs` that only calls the `register()` helpers and an `extractors.rs`. The reason is dependency shape: endpoints never depend on each other, they all depend on the shared formats and helpers, so "related code side by side" puts roles side by side. A module is still the topic; this is the layout of its HTTP surface.

**One type per file**, named after the type in snake_case. A type and its error are separate files in the same topic folder.

**One use case is one folder** in `application/src/commands/` or `application/src/queries/`. Two generic traits in `application/src/ports/` describe every use case: `CommandHandler<C>` (`async fn handle(&self, command: C) -> Result<..>`) and `QueryHandler<Q>`. A use case adds no trait of its own. The folder holds one file per role, each with a sibling `.tests.rs`:

```text
commands/record_completion/
├── mod.rs                                          # re-exports only
├── record_completion_command.rs                    # RecordCompletionCommand (data only)
├── record_completion_command_handler.rs            # RecordCompletionCommandHandler: its dependencies + impl CommandHandler<RecordCompletionCommand>
├── record_completion_command_handler.tests.rs      # handler tests, fakes from test_support
├── record_completion_command_validator.rs          # RecordCompletionCommandValidator: every input rule
└── record_completion_command_validator.tests.rs    # validator tests
```

Queries use the same shape with the `Query` suffix (`get_mood_history/`: `GetMoodHistoryQuery`, `GetMoodHistoryQueryHandler`, `GetMoodHistoryQueryValidator`). **A validator is mandatory for every command and query**, even when it checks one field: all input validation lives in `<name>_command_validator.rs` / `<name>_query_validator.rs`, never inside the handler body, so the handler reads as orchestration and the validator as the rules. There is no one-line-guard exception. Result types tuned for display and use-case helper files live in the same folder. The word "service" is reserved for `application/src/services/`, which holds implementations **shared by several use cases**; a module with none has no `services/` folder. The presentation layer depends only on the generic handler traits (generic or `Arc<dyn CommandHandler<C>>`), never on repositories or handler impls; `web-api` constructs handlers from infrastructure adapters and injects them at the composition root. That is plain constructor injection: no mediator, no registry, no runtime dispatch by message type, and no CQRS crate. `scripts/check-boundaries.sh` enforces the folder shape; folders not yet migrated are listed in `scripts/layout-usecase-baseline.txt`, a shrink-only list with its own count and ceiling.

**Tests live beside the code they cover, in their own `<name>.tests.rs` file**, attached with `#[cfg(test)] #[path = "..."] mod tests;`. Inline `#[cfg(test)] mod tests { ... }` blocks are not used: a file shows implementation, not implementation plus fixtures. Tests that exercise a crate from the outside still belong in its `tests/` directory. Every logic-bearing file has a sibling `.tests.rs`; files holding no logic are exempt — `mod.rs`/`lib.rs` re-export files, trait-only declarations, command/query/DTO data structs, and SeaORM entity/migration declarations. `scripts/check-boundaries.sh` enforces this rule (inline test blocks fail; missing siblings warn until the layout migration completes).

**A topic's `mod.rs` is its public face** — it holds the topic's own functions and constants and re-exports its types, so consumers import from the topic rather than an internal file path (`web_api::config::Config`). Moving a type between files inside a topic is therefore not a breaking change.

---

## Workspace Configuration

The root `Cargo.toml` picks up module crates by pattern, so adding a module needs no workspace edit.

```toml
[workspace]
resolver = "2"
members = [
    "src/kernel",
    "src/modules/*/domain",
    "src/modules/*/application",
    "src/modules/*/infrastructure",
    "src/modules/*/presentation",
    "src/shared/media-store",
    "src/apps/web-api",
]
default-members = ["src/apps/web-api"]

[workspace.dependencies]
# shared versions, inherited by member crates with `workspace = true`
axum = "0.8"
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
async-trait = "0.1"
```

At M0 the four `src/modules/*/...` glob lines are commented out in the real `Cargo.toml`, because Cargo rejects a member glob that matches zero directories; they activate at M1 once the first module crate exists.

`default-members` makes a bare `cargo build` or `cargo run` target the binary and its real dependency graph. Because of that, use **`cargo test --workspace`** in CI and before a push: a plain `cargo test` would only cover the default member and silently skip module tests. The Astro `web-ui` is a Node package and stays outside the Cargo workspace.

**Binary name.** `src/apps/web-api/Cargo.toml` keeps the Cargo **package** name `web-api` (so `cargo run -p web-api` and `cargo test -p web-api` keep working), but overrides the produced **binary** name to `medioss`:

```toml
[package]
name = "web-api"

[[bin]]
name = "medioss"
path = "src/main.rs"
```

`cargo build --release` therefore produces `target/release/medioss`, which is the name the deployment image and systemd-style installs invoke.

---

## Dependency Rules

**Cardinal rule: business modules never depend on one another.** No crate of one module appears in another module's manifest. This is what buys module independence and incremental build isolation, and it is the rule worth defending in review.

Within a module the order is strict:

```text
presentation    -> application -> domain -> kernel
infrastructure  -> application -> domain -> kernel
```

`infrastructure` that stores blobs may additionally depend on `media-store`. `presentation` and `infrastructure` may name their own module's `domain` directly when they map domain types to DTOs or rows, but never anything further out.

```text
                        web-api
    (mounts presentation, wires infrastructure, runs migrations)
              │                              │
  ┌───────────▼───────────┐   ┌──────────────▼────────────┐
  │ <module>-presentation │   │ <module>-infrastructure   │──► media-store
  └───────────┬───────────┘   └──────────────┬────────────┘
              │                              │
              └──────────────┬───────────────┘
                 ┌───────────▼───────────┐
                 │ <module>-application  │  owns the port traits
                 └───────────┬───────────┘
                 ┌───────────▼───────────┐
                 │   <module>-domain     │
                 └───────────┬───────────┘
                             ▼
                 ┌───────────────────────┐
                 │        kernel         │
                 └───────────────────────┘
```

`media-store` sits beside this graph, not inside it: it is used by `infrastructure` but declares no dependency on `kernel` or on any module crate (see Shared Technical Crates below). Both `presentation` and `infrastructure` converge on `application`; `application` knows neither of them.

| Crate | May depend on | Must never depend on |
| --- | --- | --- |
| `kernel` | Framework-neutral libraries only (`serde`, `uuid`, `thiserror`) | Any module crate, `media-store`, `web-api`, Axum, SeaORM |
| `<module>-domain` | `kernel`, framework-neutral libraries (`serde`, `thiserror`) | Own `application`, `infrastructure`, `presentation`; Axum, SeaORM, filesystem APIs, `media-store`, any other module |
| `<module>-application` | Own `domain`, `kernel`, framework-neutral libraries (`serde`, `thiserror`, `async-trait`, `chrono`) | Own `infrastructure`, `presentation`; Axum, SeaORM, filesystem APIs, `media-store`, any other module |
| `<module>-infrastructure` | Own `application`, own `domain`, `kernel`, SeaORM, `media-store` when it stores blobs | Own `presentation`, Axum, any other module |
| `<module>-presentation` | Own `application`, own `domain`, `kernel`, Axum, `serde` | Own `infrastructure`, SeaORM, filesystem APIs, any other module |
| `media-store` | `tokio` I/O and framework-neutral libraries | `kernel`, any module crate, Axum, SeaORM |
| `web-api` | Every module `presentation` and `infrastructure`, `media-store`, `kernel`, Axum, SeaORM, and module `application` crates where port types are needed for wiring | Nothing prohibited; it is the outermost crate |

`web-api` may name module `application` types during wiring, because injecting `Arc<dyn MeditationRepository>` requires the port trait in scope; that is composition, not business logic.

**Compile-time versus runtime direction.** Adapters point inward at compile time: `meditation-presentation` and `meditation-infrastructure` both know `meditation-application`, which knows neither. At runtime, calls flow outward through application-owned traits: a route calls a use case, which calls a port the application layer declared, dispatched to an infrastructure implementation supplied at startup. The concrete type travels in through the constructor; the source dependency never travels out.

---

## Kernel

`src/kernel` follows Domain-Driven Design's **Shared Kernel** pattern: a small, jointly owned subset of the model that several bounded contexts agree to share, together with the agreement that nobody changes it unilaterally. The crate is named `kernel` simply for brevity; the pattern and its obligations are the DDD ones. It is a liability accepted on purpose, not a convenience layer.

Initial illustrative candidates are `UserId`, `Actor` (the authenticated caller for a request), and `Role`. These are candidates, not a fixed contract: a type moves in only once a second module actually needs it, and a module that never deals with users needs none of them.

**Admission rule.** A type enters only if all three hold: at least two modules genuinely need it (one module needing it means it belongs to that module); it is a domain concept, not a helper; and it changes rarely, so anything still being designed stays in its module until it settles.

**Excluded outright:** utility and helper modules (no `utils`, `common`, `helpers`), repository and storage traits (ports belong to the consuming application layer), feature business rules, framework types (no Axum extractors, no SeaORM entities), and generic catch-all error enums. Each module defines its own domain and application error types.

**Cost of change.** `kernel` sits under every other crate, so touching it invalidates the whole dependent graph and forces a full rebuild. That pressure is the point: it makes growing the kernel expensive enough that the admission rule gets applied honestly.

---

## Shared Technical Crates

`shared/` contains reusable technical capabilities, not a Clean Architecture layer and not a business module. A crate enters `shared/` only when all of these are true:

1. At least two modules need the capability.
2. The behavior is technical and contains no feature-specific business vocabulary.
3. The crate depends on no business module.
4. Sharing it removes proven duplication rather than anticipating possible reuse.

Do not add generic `utils`, `helpers`, or `common` crates. If a capability has only one consumer, keep it in that module's `infrastructure` layer until a second consumer appears.

### Media Store

Uploaded audio is a known shared requirement, and the behavior it needs is purely technical: write a byte stream to durable storage under a generated key, open it again, delete it. That capability lives at `src/shared/media-store` so module infrastructure crates that store blobs can reuse it.

It is a **shared technical crate, not a business module**, and it does not live under `src/modules/`. It knows byte streams, opaque `StorageKey` and `StorageNamespace` values that it defines itself, and configured paths under `$MEDIOSS_DATA_DIR`. It knows nothing about `UserId`, domain vocabulary, audio semantics, or metadata, and it imports no module or `kernel` crate. A module's infrastructure layer converts its domain identifiers into a storage namespace at the boundary.

Each application layer declares its own port in its own vocabulary:

```rust
// meditation-application/src/ports/audio_store.rs
pub struct AudioUpload { pub bytes: ByteStream, pub declared_type: MediaType }
pub struct AudioLocation(pub String);          // opaque key, stored as metadata
pub struct ByteRange { pub start: u64, pub end: Option<u64> }
pub struct AudioStream { pub bytes: ByteStream, pub total_len: u64, pub content_type: MediaType }

#[async_trait]
pub trait MeditationAudioStore {
    async fn put(&self, upload: AudioUpload) -> Result<AudioLocation, StorageError>;
    async fn open(&self, location: &AudioLocation, range: Option<ByteRange>) -> Result<AudioStream, StorageError>;
    async fn delete(&self, location: &AudioLocation) -> Result<(), StorageError>;
}
```

`meditation-infrastructure` implements that trait by delegating to `media-store`. Modules that store no blobs have no `media-store` entry in their manifest at all.

---

## Composition Root and Request Flow

`src/apps/web-api/src/main.rs` is the composition root: the only place in the workspace that **selects** concrete implementations. That claim needs to be precise — `meditation-infrastructure` obviously knows SeaORM and `media-store`, since that is its job as an adapter; what it does not do is decide whether the running system uses it. Keep `main.rs` thin and let sibling modules do the work:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load()?;                        // config.rs
    let services = bootstrap::start(&config).await?;     // bootstrap.rs
    let app = router::build(state::build(services), &config);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

Startup order: load config (`MEDIOSS_DATA_DIR`, bind address, session secret, upload limits, failing fast on a missing value) → open the SeaORM connection at `$MEDIOSS_DATA_DIR/medioss.db` → run each module's migrations via `<module>_infrastructure::migrate(&db)` → construct `media-store` rooted at `$MEDIOSS_DATA_DIR/media` → construct infrastructure adapters → construct use-case services by injecting those adapters as the application-declared port traits → build `AppState`, install the `Actor` middleware, and mount each module router (`.nest("/api/meditation", meditation_presentation::router(...))`) → serve the Astro output from `web-api/static` → bind and start the server.

### Authentication context

Session resolution is a `web-api` concern, so it never becomes a cross-module dependency. Middleware in `web-api` resolves the incoming session or token through the session service it was configured with, builds an `Actor`, and inserts it into the request extensions. Module presentation crates only **read** it:

```rust
// <module>-presentation/src/extractors/actor.rs — reads, never resolves
impl<S> FromRequestParts<S> for CurrentActor {
    fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<Actor>().cloned().map(CurrentActor).ok_or(Unauthorized)
    }
}
```

Whichever module owns sessions supplies that service to `web-api` through the same constructor injection as any other dependency. No module crate depends on it. `SESSION_SECRET` actively authorizes first-run setup through a constant-time host-possession check (sent as the `X-Setup-Secret` header, checked constant-time, never stored). It is not a cookie-signing key: opaque session cookies contain random server-side IDs, so session validation uses the server-side session store instead of a signing/verifying secret.

Identity-relevant environment variables: `MEDIOSS_DATA_DIR` (database location), `SESSION_SECRET` (first-run setup possession only), `MEDIOSS_ORIGIN` (strict Origin allowlist for every state-changing browser route: setup, login, logout, user creation, password change), `SESSION_TIMEOUT` (inactivity expiry), and `SESSION_COOKIE_SECURE` (defaults to `true`; when true the cookie is named `__Host-medioss-session` with the Secure attribute — required in production over HTTPS. Setting it false for LAN HTTP switches to the plain `medioss-session` name and logs a startup warning).

### Request flow

```text
browser
  │  POST /api/meditation/sessions
  ▼
web-api router (Actor middleware) ──► meditation_presentation route
                                        │  read Actor, deserialize DTO, validate
                                        ▼
                                      meditation_application::commands::RecordSession
                                        │  load aggregate, apply meditation_domain rules
                                        ▼
                                      MeditationRepository (application-owned port)
                                        │  dispatched at runtime to
                                        ▼
                                      meditation_infrastructure::SeaMeditationRepository ──► SQLite
```

The `presentation` crate never sees SeaORM, and neither `application` nor `domain` ever sees Axum. `web-api` is the only crate with both in scope.

---

## CQRS Organization

Commands and queries are separated as an **organizational** discipline, nothing more. Commands live in `<module>-application/src/commands/` and mutate state; queries live in `<module>-application/src/queries/`, are read-only, and may return shapes tuned for display rather than domain aggregates. Each use case is one folder exporting a command/query struct, a handler trait, and a service implementing it (see _File Organization Inside a Crate_). Callers depend on the handler trait and invoke it directly; there is no mediator, no handler registry, and no runtime dispatch by message type. Modules have no event bus and no background-job framework today; the one cross-module reaction (user deletion cleanup) is a listener port resolved at the composition root. If a second genuine producer/consumer pair or a recurring job appears, add `application/src/events/` (one integration event + handler per folder, `Requests/`-style) or `application/src/jobs/` as a deliberate decision then — not before.

**Known deviation: raw ports in meditation presentation.** The rule is that presentation depends only on handler traits. Four sites in `meditation-presentation` still read a port directly, because the work they do is streaming or request-shaped rather than a use case: `cover_validator` (normalizing a streamed cover upload before the command runs), `store` (streaming audio bytes in `audio_stream.rs`), and `programs` plus `playback` (both in `programs.rs`, decorating a program with its cover and progress). They are tracked, not endorsed; a later change should give each a handler-shaped use case. Nothing else in presentation reads a repository.

---

## Favorite Meditations API

The `meditation` module stores each user's favorites in `meditation_favorites` (`user_id`, `meditation_id`, `created_at`; primary key on the first two). The routes need a session. The two write routes also check the `Origin` header.

| Route | Result |
| --- | --- |
| `PUT /api/meditation/{id}/favorite` | `204`. Idempotent. `404` if the meditation does not exist. |
| `DELETE /api/meditation/{id}/favorite` | `204`. Idempotent: `204` even if it was not a favorite. `404` if the meditation does not exist. |
| `GET /api/meditation/favorites` | `200`, a JSON array of the caller's favorite meditations, most recently favorited first. Each item has the `MeditationResponse` shape with `"favorite": true`. |

Every `MeditationResponse` that is made for the caller (library, daily suggestion, detail, program members) has a boolean `favorite`. The upload and description-update replies always send `"favorite": false`. Deleting a meditation or a user removes the matching favorite rows.

Known limit: export/import does not carry favorites. The portability module is not changed.

## Runtime Data and Media Security

No runtime state lives in the source tree. Everything mutable lives under `$MEDIOSS_DATA_DIR`, supplied by the environment and mounted as a volume in Docker.

```text
$MEDIOSS_DATA_DIR/
├── medioss.db                  # SQLite: all module tables
└── media/
    ├── meditation/             # program folders: audio (Ogg Opus) + covers
    │   └── {program_folder}/
    ├── soundscape/             # built-in sound audio and icons
    └── identity/               # per-user avatars
        └── {user_id}/
```

Each module owns one `media/` root (`media/meditation/`, `media/soundscape/`, `media/identity/{user_id}/`), so a per-module backup or wipe never needs a path list. Stored files keep opaque names; titles, MIME types, and tags stay database metadata. Startup migrates a pre-1.0.0 flat `media/` layout once, renaming each legacy folder into its module root (never deleting). Nothing is written under `src/`, and there is no `web-api/media` directory. Backing up `$MEDIOSS_DATA_DIR` captures everything.

**One database, module-owned schemas.** All modules share one SQLite file, but each module owns its tables exclusively and ships its own migrations in its `infrastructure` crate. Two rules keep that from collapsing into a shared schema:

- Each module's migrator uses a **distinct ledger table** (for example `meditation_migrations`), so migrations version independently and no module's history blocks another's.
- **No cross-module foreign keys.** A module may store a `UserId` as a plain column, but it declares no FK into another module's table. References across modules are by value, validated by the owning module, never by the database.

**Opaque keys.** Stored files are named by a generated identifier; original filenames, titles, MIME types, durations, and tags are database metadata, never path segments. Because the key is generated rather than derived from user input, path traversal is structurally impossible: no attacker-controlled string reaches the resolved path.

**User media is never a static file.** Every read goes through an authenticated route:

```text
GET /api/meditation/{id}/audio
  1. read Actor from request extensions; reject unauthenticated requests
  2. load metadata from the database by id
  3. verify metadata.owner_id == actor.user_id, else 404 (not 403)
  4. open the byte stream through the application-owned storage port, passing
     any Range header through as ByteRange
  5. stream 200 or 206 with the DB-recorded content type
```

Ownership is checked against database metadata on every request, so a leaked or guessed identifier is not enough to read another user's audio. Returning `404` rather than `403` avoids confirming that the resource exists.

**Upload hardening.** Enforce the body size limit **before** buffering (Axum's `DefaultBodyLimit`, so an oversized request is rejected without being read into memory or written to disk), validate the **actual** media type from decoded content rather than the declared `Content-Type` or filename extension, write to a temporary path and move into place only after validation succeeds, and store the verified content type and duration as metadata.

**Static assets are a separate concern.** The Astro build output is copied to `src/apps/web-api/static/`. That directory is excluded from version control by `.gitignore` as a **build artifact**, not runtime state. It may be embedded in the binary or served publicly; it never contains user uploads.

---

## Frontend Stack

The `web-ui` app is **Astro with TypeScript**, using **Svelte 5** (via `@astrojs/svelte`) for interactive islands and **Tailwind CSS** with **shadcn-svelte** for UI components.

Astro renders every page as static HTML; only genuinely interactive components — the meditation audio player, the soundscape mixer, mood forms and charts, admin controls — hydrate as Svelte islands with `client:*` directives. Cross-island state (e.g. a globally visible player) lives in shared `.svelte.ts` rune modules or nanostores, not in a top-level framework app.

Svelte was chosen over SolidJS deliberately: its compiler emits the smallest per-island runtime (roughly 2 KB gzip versus 8 KB), Svelte 5 with runes is stable while Solid 2.0 is still pre-release, and the ecosystem covers this project's needs directly (shadcn-svelte for accessible components, chart libraries for mood analytics). shadcn-svelte components are copied into the repository rather than installed as a dependency, which fits the self-hosted, no-external-runtime-dependency posture.

**Frontend conventions.**

- Each page mounts a single Svelte island (for example `client:only="svelte"` in `pages/stats.astro`), which may render the app shell (`lib/components/shell/app-shell.svelte`) around the screen. It is still a multi-page app: every route is its own Astro page, and there is no client router.
- Pure TypeScript logic is tested with `bun test` in a sibling `<name>.test.ts` file. End-to-end tests are `*.e2e.ts` files under `src/apps/web-ui/e2e/`, run by Playwright (`playwright.config.ts`). E2e is deliberately **minimal**: the default run is a short core suite (`e2e/core/`) that covers only the flows most likely to break — sign-in gating, one flow per major screen, playback controls. Exhaustive per-requirement suites are not written for routine changes. Retired suites live under `e2e/extended/`, excluded from `testMatch` via `testIgnore`; they can be restored temporarily by moving files back under `e2e/seeded/` when a deep regression pass is wanted.
- UI text lives in co-located `*.messages.ts` files with `en` and `tr` values.
- The only client-side storage is the language preference (`localStorage` key `medioss.locale`).

---

## Incremental Build Isolation

Cargo tracks dependencies per crate. Changing a crate's source invalidates that crate and its **reverse dependencies**; everything else is reported `Fresh`. Because modules never depend on one another, a feature change stays inside one module plus the binary — and because the layers are separate crates, most changes stay inside one layer.

| Change | Rebuilt | Fresh |
| --- | --- | --- |
| `meditation-domain` | `meditation-domain`, `meditation-application`, `meditation-infrastructure`, `meditation-presentation`, `web-api` | `kernel`, `media-store`, every other module's crates |
| `meditation-application` | `meditation-application`, `meditation-infrastructure`, `meditation-presentation`, `web-api` | `meditation-domain`, `kernel`, `media-store`, every other module's crates |
| `meditation-infrastructure` | `meditation-infrastructure`, `web-api` | everything else, including `meditation-domain`, `meditation-application`, `meditation-presentation` |
| `meditation-presentation` | `meditation-presentation`, `web-api` | everything else, including `meditation-domain`, `meditation-application`, `meditation-infrastructure` |
| `media-store` | `media-store`, the infrastructure crates that use it, `web-api` | `kernel`, all `domain`, `application` and `presentation` crates, modules that store no blobs |
| `kernel` | Every dependent crate in the workspace | Third-party dependencies only |

**A change rebuilds reverse dependencies, not the workspace.** Editing `meditation-application` leaves sibling modules genuinely untouched — and leaves `meditation-domain` untouched too — with Cargo reporting them `Fresh`. This is the concrete payoff of the cardinal rule, and it degrades the moment a cross-module dependency is added. Likewise, `cargo test -p meditation-domain` compiles `kernel` and `meditation-domain` only, and `cargo check -p meditation-presentation` never pulls in SeaORM, because `presentation` does not depend on `infrastructure`.

---

## Testing Strategy and Boundary Enforcement

| Level | Location | Dependencies | What it covers |
| --- | --- | --- | --- |
| Domain unit tests | `<module>-domain` | `kernel` only | Entities, value objects, invariants, domain errors. Pure functions, no I/O, no fakes. |
| Application unit tests | `<module>-application` | Own `domain` plus in-memory port fakes | Commands, queries, use-case orchestration, authorization decisions. No I/O, no database. |
| Infrastructure integration tests | `<module>-infrastructure/tests/` | Temporary SQLite file and temporary directory, removed on drop | Migrations apply cleanly, mappers round-trip, repository queries return the right rows, storage adapters move real bytes. |
| Presentation router tests | `<module>-presentation/src/**.tests.rs` beside the handlers they cover | Axum test client, mocked application services, `Actor` inserted directly into extensions | Routing, extractors, DTO serialization, status codes, auth guards, error mapping. No database. |
| Smoke and E2E tests | `web-api/tests/` | Full binary, temporary data directory pointed to by `MEDIOSS_DATA_DIR` | End-to-end request flow through the real composition root: config loading, migrations, router, and every mounted module. |

Run them with `cargo test --workspace`; `default-members` would otherwise limit a bare `cargo test` to the binary and silently skip every module.

**Boundary enforcement.** The primary enforcement mechanism is the Cargo dependency graph itself: a forbidden dependency (for example `meditation-domain` depending on `axum`, or one module depending on another) is a compile-time manifest error, not a lint warning. A CI check adds a second, explicit layer using `cargo metadata --format-version 1` to assert:

1. No `*-domain` or `*-application` crate lists `axum`, `sea-orm`, or any filesystem-access crate as a dependency.
2. No `*-presentation` crate lists `sea-orm` or a filesystem-access crate.
3. No module crate lists another module's crate.
4. `media-store` and `kernel` list no crate under `src/modules/`.

### Sibling-test rule

`scripts/check-boundaries.sh` fails when a logic-bearing source file has no sibling `<name>.tests.rs` containing at least one real assertion and wired by the three consecutive line-start lines `#[cfg(test)]`, `#[path = "<name>.tests.rs"]`, `mod tests;`. There is no exemption list: a file either has tests or is exempt by one of the proven kinds below. An empty `.tests.rs`, a hook inside a comment or raw string, and tests made only of literal tautologies (`assert!(true)`, `assert_eq!(1, 1)`) do not satisfy it.

Exempt by kind, each proven mechanically because it is provably logic-free or test-only:

- **A** Files in a `router_tests/` directory declared by name from its `mod.rs`, when `lib.rs` gates `router_tests` with a literal `#[cfg(test)]`.
- **B** SeaORM entity files under `persistence/entities/` or named `entity.rs` with no `fn`, no `macro_rules!` and no `impl` other than `impl ActiveModelBehavior for ActiveModel {}`.
- **C** `test_support.rs` / `test_support/` declared exactly once as `mod test_support;` directly under `#[cfg(test)]` (or the cross-crate fakes gate `#[cfg(any(test, feature = "test-support"))]`, which also requires the feature to be dev-only), with no `#[path]` or stacked `#[cfg*]` attribute, no mention elsewhere in the crate root, and no production file using it.
- **D** A crate's `src/main.rs` that holds only `fn main` and `fn run` plus declarations and data: no other fn, no `impl`, `static`, `macro_rules!` or inline `mod x { }`. Startup logic lives in `web_api::bootstrap`, with tests.

`mod.rs` and `lib.rs` are skipped by the sibling rule but ratcheted by `scripts/layout-modrs-baseline.txt`: a `mod.rs`/`lib.rs` holding `fn`, `impl`, `static` or `macro_rules!` that is not in the baseline fails, and the baseline (with its own `# count` and `MODRS_CEILING`) may only shrink.

Residual gaps are review-enforced, not machine-enforced: an assertion token inside a string literal or tautologies on variables (`assert_eq!(v, v)`) still satisfy the floor, the assertion check is file-wide rather than per test, and the baseline `# count` and `MODRS_CEILING` can be edited together in one diff. Reviewers reject any diff that raises either number.

Environment variables read by code under test are set by exactly one test each, through a drop guard that removes the variable even when an assertion fails; tests that share a variable race because the test harness runs them in parallel.
