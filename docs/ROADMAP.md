# Roadmap: MediOSS Wellness App

This roadmap turns the [PRD's implementation phases](./PRD.md#implementation-phases) into ordered, verifiable milestones aligned with the [module-first architecture](./ARCHITECTURE.md). Each milestone lists its deliverables and an explicit exit criterion; a milestone is done when its exit criterion passes, not when its code merges.

Business modules were created on demand, milestone by milestone, following the `src/modules/<module>/{domain,application,infrastructure,presentation}` pattern. Five exist as of v1.1.1: `identity`, `meditation`, `mood-tracker`, `portability` and `soundscape` (see [ARCHITECTURE.md](./ARCHITECTURE.md)). Milestones M0-M9 are delivered, v1.1.1 is the current release, and Phase 3 is open.

```text
M0 ── M1 ── M2 ── M3 ── M4                  Phase 1: MVP
                       │
                       └── M5 ── M6 ── M7 ── M8   Phase 2: Enhanced
                                                 │
                                                 └── M9 ── v1.0.0          Release

M10+                                           Phase 3: Polish & Expansion
```

Execution order (scope decision, 2026-10-03; renumbered so numbers follow execution): M5 (multi-user) → M6 (library depth) → M7 (data portability) → M8 (UI design) → M9 (release). Milestone numbers were renumbered on 2026-10-03; older specs may still reference the previous numbering.

---

## Phase 1: MVP (delivered)

Goal: a single-user, self-hosted instance where one person can register, upload meditation audio, mix white noise, and log moods — deployed with one `docker compose up`.

### M0 — Workspace Skeleton (delivered)

The empty workspace that proves the architecture compiles before any feature exists.

- Root `Cargo.toml` workspace with member globs from [ARCHITECTURE.md: Workspace Configuration](./ARCHITECTURE.md#workspace-configuration)
- `src/kernel` crate (empty placeholder types)
- `src/apps/web-api` binary (`medioss`) serving `GET /api/health`
- `src/apps/web-ui` Astro scaffold with one page, with `@astrojs/svelte`, Tailwind, and shadcn-svelte initialized ([ARCHITECTURE.md: Frontend Stack](./ARCHITECTURE.md#frontend-stack))
- `$MEDIOSS_DATA_DIR` config loading that fails fast when missing
- Dockerfile (3-stage) and `docker-compose.yml` matching [DEPLOYMENT.md](./DEPLOYMENT.md)
- CI: `cargo test --workspace`, `cargo clippy`, the dependency-boundary check from [ARCHITECTURE.md: Testing Strategy](./ARCHITECTURE.md#testing-strategy-and-boundary-enforcement), and an Astro build

**Exit criterion:** `docker compose up` serves the health endpoint and the placeholder UI from one container; CI is green on an empty feature set.

### M1 — Identity Module (delivered)

First real module; every later module depends on its `Actor` context. Delivered with admin-driven registration (no public signup), Argon2id hashing, opaque server-side sessions, per-account exponential login throttling, first-run setup gated by host possession of `SESSION_SECRET`, and the `medioss reset-password` operator CLI.

- `identity` module: registration, login, password hashing (bcrypt/argon2), session issuance and verification
- `kernel` gains `UserId`, `Actor`, `Role` (its first legitimate tenants)
- `web-api` middleware: session → `Actor` in request extensions
- First-run flow: instance starts with an admin account setup
- CLI operator recovery: `medioss reset-password <email>` works against the local database without a running server (PRD: Operator Recovery)
- Rate limiting on login attempts
- Session timeout from `SESSION_TIMEOUT`

**Exit criterion:** E2E test — register, log in, call an authenticated endpoint, get 401 without a session. Runs in CI against the real binary. CLI reset restores access to a locked-out admin account.

### M2 — Meditation Module (delivered)

Implements the module documented throughout ARCHITECTURE.md.

- `src/shared/media-store` crate: put/open/delete by opaque key under `$MEDIOSS_DATA_DIR/media`
- `meditation` module: upload (100MB limit enforced pre-buffer, real content type validation), library list, delete
- Authenticated, ownership-checked audio streaming with Range support (200/206), per [ARCHITECTURE.md: Runtime Data and Media Security](./ARCHITECTURE.md#runtime-data-and-media-security)
- Playback UI: play/pause/seek/speed (0.5x–2.0x), resume from last position
- Session completion recording (start/finish timestamps)

**Exit criterion:** E2E test — upload an MP3, stream it back with a Range header as the owner, receive 404 for another user's file. Playback works in a real browser.

### M3 — Soundscape Module (delivered)

- `soundscape` module: upload custom loops (reusing `media-store`), sound library
- Mixer: 3 simultaneous sounds with per-sound volume
- Sleep timer with fade-out
- Continuous playback ≥ 1 hour without gaps in a real browser

**Exit criterion:** manual QA script — mix 3 sounds, set a fade timer, verify smooth fade; automated tests cover upload/list/stream ownership.

### M4 — Mood Tracker Module (delivered)

- `mood-tracker` module: 1–10 scale log with emotion/activity tags and optional note
- Seeded tag list (predefined emotions and activities)
- Daily log view and simple history list
- Optional post-meditation mood prompt (frontend composition of two module APIs — no cross-module Rust dependency)

**Exit criterion:** E2E test — create a mood log in under 3 requests; history returns only the caller's entries.

---

## Phase 2: Enhanced Features (delivered)

Goal: multi-user instances become pleasant to run and richer to use.

### M5 — Multi-User & Admin (delivered)

- Admin dashboard: list/disable/remove users
- Account and data deletion: user-initiated and admin-initiated, wiping all database records and `media/{user_id}/` (PRD: Account & Data Deletion)
- ~~Per-user storage quotas~~ — not delivered in v1.0.0; deferred to Phase 3 (no quota code exists)
- Profile settings: display name, avatar, password change

**Exit criterion:** two concurrent users cannot see each other's media, moods, or sessions — verified by an automated isolation test suite. Deleting an account leaves zero database rows and zero media files for that user.

### M6 — Library & Mixer Depth (delivered)

- Meditation: full-text search over titles and categories (tags and duration filters remain maybe-level; see M10+)
- Soundscape: unlimited simultaneous sounds, saved mixes, 5–120 min fade timers (delivered by spec 0003)
- Player: Media Session API hardware/lock-screen controls **Exit criterion:** search returns correct results over a 100+ item library; hardware media keys control playback on a real device.

### M7 — Data Portability (delivered)

- Mood trends: weekly/monthly charts (delivered by spec 0006)
- CSV/JSON export and import of the caller's own data (lossless round-trip, enabling instance migration)

**Exit criterion:** export → import round-trip on a fresh instance reproduces the original data.

### M8 — UI Design (Full Design Pass) (delivered)

Closes Phase 2. Until it ships, the UI stays at functional basic level — no design work lands earlier.

- Visual design implemented from scratch across every flow (auth, home, meditation, soundscape, mood, statistics, admin, profile settings)
- Mobile-responsive pass across all flows (PRD design principles) — absorbed from M9 (Hardening)
- Error, empty, and loading states across the UI — absorbed from M9 (Hardening)

**Exit criterion:** every user flow carries the final design and is usable from a 360 px viewport up; a manual walkthrough of each flow on a real mobile and desktop browser shows no unstyled or broken states.

---

## Release: v1.0.0 (delivered)

Runs after M8; operational only.

### M9 — MVP Hardening & Release (delivered)

- Backup/restore procedure tested against a populated `$MEDIOSS_DATA_DIR`
- Seed-content documentation (where operators can source public-domain audio)
- Version `1.0.0` tag, release image (`ghcr.io/ahmet-cetinkaya/medioss`), and a validated [DEPLOYMENT.md](./DEPLOYMENT.md) walkthrough on a clean VPS

**Exit criterion:** PRD Phase 1 checklist fully checked; a non-author deploys a working instance from the docs alone in under 5 minutes.

---

## Phase 3: Polish & Expansion

Goal: long-tail quality and optional integrations. Scope here is intentionally loose and re-planned after Phase 2 ships.

### M10+ — Candidates (ordered by current expectation, not commitment)

- Accessibility audit to WCAG 2.1 AA and fixes
- Daily reminder notifications (in-app/PWA local only, per PRD Non-Goals)
- Advanced insights (meditation ↔ mood correlation views)
- Aggregate, non-content admin instance insight (active user counts, storage use; never another user's content) — moved from M5 (previously M6) by user decision 2026-09-30
- Per-user storage quotas (deferred from M5)
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
