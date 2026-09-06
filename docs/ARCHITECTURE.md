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

This is a **draft**. Final business-module boundaries are decided feature by feature as the product is built. `meditation` is used throughout as the single worked example; it is not a complete module inventory, and the number of modules is deliberately left open.

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

Every business module is four independent Cargo crates, one per Clean Architecture layer:

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
    │   │   │       ├── ports/              # MeditationRepository, MeditationAudioStore
    │   │   │       ├── commands/           # UploadMeditation, RecordSession
    │   │   │       ├── queries/            # SearchLibrary, GetStreak
    │   │   │       ├── services/           # use-case services, module facade
    │   │   │       └── error.rs            # application errors
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
    │   │           ├── lib.rs              # pub fn router(...) -> Router
    │   │           ├── routes/
    │   │           ├── extractors/
    │   │           └── dto/
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

**Group by domain topic.** A crate's `src/` is organized into folders named after the concern they own, not the kind of construct they hold. A topic's type, its error, the functions operating on it, and its tests live together. There are no `models/`, `types/`, `errors/`, or `utils/` buckets — splitting one concern across category folders is what this rule exists to prevent.

**One type per file**, named after the type in snake_case. A type and its error are separate files in the same topic folder.

**Tests live beside the code they cover, in their own `<name>.tests.rs` file**, attached with `#[cfg(test)] #[path = "..."] mod tests;`. Inline `#[cfg(test)] mod tests { ... }` blocks are not used: a file shows implementation, not implementation plus fixtures. Tests that exercise a crate from the outside still belong in its `tests/` directory.

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

Whichever module owns sessions supplies that service to `web-api` through the same constructor injection as any other dependency. No module crate depends on it. Whether that service issues a signed session cookie or a JWT is a decision left to that module; `web-api` only needs a signing/verifying secret from configuration (currently named `JWT_SECRET`, see [DEPLOYMENT.md](./DEPLOYMENT.md)) to hand to whichever mechanism is chosen.

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

Commands and queries are separated as an **organizational** discipline, nothing more. Commands live in `<module>-application/src/commands/` and mutate state; queries live in `<module>-application/src/queries/`, are read-only, and may return shapes tuned for display rather than domain aggregates. Callers invoke the service function directly; there is no mediator, no handler registry, and no runtime dispatch by message type.

---

## Runtime Data and Media Security

No runtime state lives in the source tree. Everything mutable lives under `$MEDIOSS_DATA_DIR`, supplied by the environment and mounted as a volume in Docker.

```text
$MEDIOSS_DATA_DIR/
├── medioss.db                  # SQLite: all module tables
└── media/
    └── {user_id}/
        └── {opaque_id}         # no extension, no original filename
```

Nothing is written under `src/`, and there is no `web-api/media` directory. Backing up `$MEDIOSS_DATA_DIR` captures everything.

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
| Presentation router tests | `<module>-presentation/tests/` | Axum test client, mocked application services, `Actor` inserted directly into extensions | Routing, extractors, DTO serialization, status codes, auth guards, error mapping. No database. |
| Smoke and E2E tests | `web-api/tests/` | Full binary, temporary data directory pointed to by `MEDIOSS_DATA_DIR` | End-to-end request flow through the real composition root: config loading, migrations, router, and every mounted module. |

Run them with `cargo test --workspace`; `default-members` would otherwise limit a bare `cargo test` to the binary and silently skip every module.

**Boundary enforcement.** The primary enforcement mechanism is the Cargo dependency graph itself: a forbidden dependency (for example `meditation-domain` depending on `axum`, or one module depending on another) is a compile-time manifest error, not a lint warning. A CI check adds a second, explicit layer using `cargo metadata --format-version 1` to assert:

1. No `*-domain` or `*-application` crate lists `axum`, `sea-orm`, or any filesystem-access crate as a dependency.
2. No `*-presentation` crate lists `sea-orm` or a filesystem-access crate.
3. No module crate lists another module's crate.
4. `media-store` and `kernel` list no crate under `src/modules/`.
