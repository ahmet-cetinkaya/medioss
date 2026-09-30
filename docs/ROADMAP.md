# Roadmap: MediOSS Wellness App

This roadmap turns the [PRD's implementation phases](./PRD.md#implementation-phases) into ordered, verifiable milestones aligned with the [module-first architecture](./ARCHITECTURE.md). Each milestone lists its deliverables and an explicit exit criterion; a milestone is done when its exit criterion passes, not when its code merges.

Business modules are created on demand, milestone by milestone, following the `src/modules/<module>/{domain,application,infrastructure,presentation}` pattern. Module names below are working names; final boundaries are decided when each module is built (see the draft note in ARCHITECTURE.md).

```text
M0 ── M1 ── M2 ── M3 ── M4 ── M5      Phase 1: MVP
                              │
                              ├── M6 ── M7 ── M8    Phase 2: Enhanced
                              │
                              └────────── M9+       Phase 3: Polish & Expansion
```

---

## Phase 1: MVP

Goal: a single-user, self-hosted instance where one person can register, upload meditation audio, mix white noise, and log moods — deployed with one `docker-compose up`.

### M0 — Workspace Skeleton

The empty workspace that proves the architecture compiles before any feature exists.

- Root `Cargo.toml` workspace with member globs from [ARCHITECTURE.md: Workspace Configuration](./ARCHITECTURE.md#workspace-configuration)
- `src/kernel` crate (empty placeholder types)
- `src/apps/web-api` binary (`medioss`) serving `GET /api/health`
- `src/apps/web-ui` Astro scaffold with one page, with `@astrojs/svelte`, Tailwind, and shadcn-svelte initialized ([ARCHITECTURE.md: Frontend Stack](./ARCHITECTURE.md#frontend-stack))
- `$MEDIOSS_DATA_DIR` config loading that fails fast when missing
- Dockerfile (3-stage) and `docker-compose.yml` matching [DEPLOYMENT.md](./DEPLOYMENT.md)
- CI: `cargo test --workspace`, `cargo clippy`, the dependency-boundary check from [ARCHITECTURE.md: Testing Strategy](./ARCHITECTURE.md#testing-strategy-and-boundary-enforcement), and an Astro build

**Exit criterion:** `docker-compose up` serves the health endpoint and the placeholder UI from one container; CI is green on an empty feature set.

### M1 — Identity Module (delivered)

First real module; every later module depends on its `Actor` context. Delivered with admin-driven registration (no public signup), Argon2id hashing, opaque server-side sessions, per-account exponential login throttling, first-run setup gated by host possession of `SESSION_SECRET`, and the `medioss reset-password` operator CLI.

- `identity` module: registration, login, password hashing (bcrypt/argon2), session issuance and verification
- `kernel` gains `UserId`, `Actor`, `Role` (its first legitimate tenants)
- `web-api` middleware: session → `Actor` in request extensions
- First-run flow: instance starts with an admin account setup
- CLI operator recovery: `medioss admin reset-password <email>` works against the local database without a running server (PRD: Operator Recovery)
- Rate limiting on login attempts
- Session timeout from `SESSION_TIMEOUT`

**Exit criterion:** E2E test — register, log in, call an authenticated endpoint, get 401 without a session. Runs in CI against the real binary. CLI reset restores access to a locked-out admin account.

### M2 — Meditation Module (worked example becomes real)

Implements the module documented throughout ARCHITECTURE.md.

- `src/shared/media-store` crate: put/open/delete by opaque key under `$MEDIOSS_DATA_DIR/media`
- `meditation` module: upload (100MB limit enforced pre-buffer, real content type validation), library list, delete
- Authenticated, ownership-checked audio streaming with Range support (200/206), per [ARCHITECTURE.md: Runtime Data and Media Security](./ARCHITECTURE.md#runtime-data-and-media-security)
- Playback UI: play/pause/seek/speed (0.5x–2.0x), resume from last position
- Session completion recording (start/finish timestamps)

**Exit criterion:** E2E test — upload an MP3, stream it back with a Range header as the owner, receive 404 for another user's file. Playback works in a real browser.

### M3 — Soundscape Module

- `soundscape` module: upload custom loops (reusing `media-store`), sound library
- Mixer: 3 simultaneous sounds with per-sound volume
- Sleep timer with fade-out
- Continuous playback ≥ 1 hour without gaps in a real browser

**Exit criterion:** manual QA script — mix 3 sounds, set a fade timer, verify smooth fade; automated tests cover upload/list/stream ownership.

### M4 — Mood Module

- `mood` module: 1–10 scale log with emotion/activity tags and optional note
- Seeded tag list (predefined emotions and activities)
- Daily log view and simple history list
- Optional post-meditation mood prompt (frontend composition of two module APIs — no cross-module Rust dependency)

**Exit criterion:** E2E test — create a mood log in under 3 requests; history returns only the caller's entries.

### M5 — MVP Hardening & Release

- Mobile-responsive pass across all flows (PRD design principles)
- Backup/restore procedure tested against a populated `$MEDIOSS_DATA_DIR`
- Error states, empty states, and loading states in the UI
- Seed-content documentation (where operators can source public-domain audio)
- Version `0.1.0` tag, release image, and a validated [DEPLOYMENT.md](./DEPLOYMENT.md) walkthrough on a clean VPS

**Exit criterion:** PRD Phase 1 checklist fully checked; a non-author deploys a working instance from the docs alone in under 5 minutes.

---

## Phase 2: Enhanced Features

Goal: multi-user instances become pleasant to run and richer to use.

### M6 — Multi-User & Admin

- Admin dashboard: list/disable/remove users
- Account and data deletion: user-initiated and admin-initiated, wiping all database records and `media/{user_id}/` (PRD: Account & Data Deletion)
- Per-user storage quotas
- Profile settings: display name, avatar, password change

**Exit criterion:** two concurrent users cannot see each other's media, moods, or sessions — verified by an automated isolation test suite. Deleting an account leaves zero database rows and zero media files for that user.

### M7 — Library & Mixer Depth

- Meditation: full-text search over titles and categories (tags and duration filters remain maybe-level; see M9+)
- Soundscape: unlimited simultaneous sounds, saved mixes, 5–120 min fade timers (delivered by spec 0003)
- Player: Media Session API hardware/lock-screen controls

**Exit criterion:** search returns correct results over a 100+ item library; hardware media keys control playback on a real device.

### M8 — Data Portability

- Mood trends: weekly/monthly charts (delivered by spec 0006)
- CSV/JSON export and import of the caller's own data (lossless round-trip, enabling instance migration)

**Exit criterion:** export → import round-trip on a fresh instance reproduces the original data.

---

## Phase 3: Polish & Expansion

Goal: long-tail quality and optional integrations. Scope here is intentionally loose and re-planned after Phase 2 ships.

### M9+ — Candidates (ordered by current expectation, not commitment)

- Accessibility audit to WCAG 2.1 AA and fixes
- Daily reminder notifications (in-app/PWA local only, per PRD Non-Goals)
- Advanced insights (meditation ↔ mood correlation views)
- Aggregate, non-content admin instance insight (active user counts, storage use; never another user's content) — moved from M6 by user decision 2026-09-30
- Meditation tag system and duration filters
- Notification preferences (in-app reminders; local only, per PRD Non-Goals)
- Performance pass: startup time, memory ceiling, 10k+ mood entries

**Exit criterion:** re-scoped at Phase 2 completion; each adopted item gets its own milestone with an explicit criterion before work starts.

---

## Working Agreements

- **One module per milestone.** A milestone that touches two modules is two milestones.
- **Exit criteria are tests where possible.** Anything called "done" has a CI test, an E2E script, or a documented manual QA procedure that passed.
- **Architecture rules hold from M0.** The dependency-boundary CI check ships before the first feature, so violations are impossible to accumulate.
- **Docs move with code.** A milestone that changes deployment or API behavior updates [DEPLOYMENT.md](./DEPLOYMENT.md) / [ARCHITECTURE.md](./ARCHITECTURE.md) in the same change.
- **Re-plan at phase boundaries.** Timeboxes come from the PRD (Phase 1: 8–12 weeks; Phases 2–3: ~8 weeks each) but milestone contents are re-cut at each phase boundary based on what shipped.
