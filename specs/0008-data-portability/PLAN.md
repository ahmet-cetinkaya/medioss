---
status: approved
spec: specs/changes/0008-data-portability/SPEC.md
created: 2026-10-03
---

# Plan — Data Portability (Spec 0008, M7)

## Goal

Ship the export/import capability of spec 0008: every signed-in user can download their own data as JSON (canonical round-trip) or CSV (mood-only), and import a previously exported file into an instance with duplicate detection, meditation matching by name+duration, atomic validate-then-write, and per-kind result counts. Exit: roadmap M7's criterion — export → import on a fresh instance reproduces the original data.

## Context

- No export/import code exists today (verified by grep, 2026-10-03).
- Composition root: `src/apps/web-api/src/router.rs:13-48` builds each module's presentation router with its own `SharedDependencies` and merges into `/api`. Cross-module data flows through ports defined by the consumer and adapted in the composition root — precedent: `MeditationTimelineSource` (`src/modules/mood-tracker/application/src/ports/meditation_timeline_source.rs:21-23`) implemented by `MeditationTimelineAdapter` (`src/apps/web-api/src/mood_tracker.rs:26-33`).
- Existing write surfaces (no bulk variants anywhere):
  - Mood: `MoodEntryRepository::insert(user_id, score, note, recorded_at)`, `list_all_for_user` (`src/modules/mood-tracker/application/src/ports/mood_entry_repository.rs:14-36`).
  - Meditation: `SessionRepository::record_if_new(user, meditation_id, started_at, finished_at)` (idempotent), `NoteRepository::insert(user, meditation_id, text, created_at)`, `PlaybackStateRepository::upsert_position` / `set_completed` (`src/modules/meditation/application/src/ports/`).
- File conventions: multipart via `read_field_capped` (`meditation/presentation/src/response.rs:14`); downloads set attachment + sanitized filename (`handlers_audio.rs:312`).
- UI host: `settings.svelte` (mounted from `settings.astro`); upload pattern to copy from `admin.svelte:445-474`.
- Tests: router-test pattern from `meditation/presentation/src/router.tests.rs` (TestApp + fakes + oneshot). Project conventions: one definition per file, `.tests.rs` siblings, C#-style folders, English content.
- Boundary script `scripts/check-boundaries.sh:34` allowlists neutral deps for domain/application; presentation crates are unrestricted.

## Decisions

- **D1 — New module `src/modules/portability/`** (domain/application/infrastructure/presentation), following module-first architecture. It defines its own ports for the data it touches (`OwnMoodRecords` + `MoodRecordSink`, `OwnPracticeRecords` + `PracticeSink`, `MeditationLookup`); the composition root implements adapters over the existing SeaORM repositories — same pattern as `MeditationTimelineAdapter`. No module-to-module Cargo dependency; boundary script stays green.
- **D2 — Formats live in presentation.** Application exchanges typed DTOs (`ExportDocument`, `MoodRecord`, `PracticeRecord`, `ImportReport`). Presentation serializes: JSON via `serde_json`, CSV via the `csv` crate (RFC 4180 quoting). No new application-layer dependency → no `check-boundaries.sh` change needed.
- **D3 — No new bulk repository methods.** Import loops existing writes after full validation (validate-then-write). Personal-data scale (hundreds of rows) makes this acceptable; a mid-write infrastructure failure surfaces as an error (partial writes possible only on infra failure, never on validation failure — FR-008 satisfied).
- **D4 — Duplicate detection** builds a set of `(recorded_at, score, note)` from `list_all_for_user` before writing; exact-triple matches are skipped (FR-007).
- **D5 — Meditation matching** is exact on `(name, duration_ms)`. If a target instance has several meditations sharing both, the lowest id wins (deterministic). No match → record skipped and counted (FR-009/010).
- **D6 — Timestamps and CSV shape.** All times RFC 3339 UTC. CSV columns: `recorded_at,score,note`; empty note → empty field; UTF-8. JSON carries `format_version: 1`; unknown version → reject (FR-008).
- **D7 — Routes** under the portability presentation router, merged at `/api/portability`: `GET /portability/export/json`, `GET /portability/export/csv` (attachment downloads), `POST /portability/import` (multipart, `read_field_capped`). Auth: `CurrentActor` — caller's own data only (FR-013).
- **D8 — Constitution check (project instructions):** module-first layout respected (D1); no drive-by refactors; repos untouched (D3); UI stays functional-basic (M8 owns design); safety boundaries unaffected (self-data only).

## Scope

**In:** portability module (ports, DTOs, application export/import use cases, presentation routes), web-api adapters + router merge, settings.svelte Data section (export buttons, import upload, report display), unit/router/round-trip tests, gate + live QA.

**Out (per spec OOS):** library migration, admin bulk export, cloud sync, replace-all import, deletion flows, chart changes.

## Tasks

1. **Module skeleton + ports + DTOs** (deps: none)
   - `src/modules/portability/{domain,application,infrastructure,presentation}` crates wired into the workspace; kernel-only domain placeholder.
   - Ports (application/src/ports/, one per file): `OwnMoodRecords::list_for_user`, `MoodRecordSink::insert`, `OwnPracticeRecords::list_for_user`, `PracticeSink::{record_session, insert_note, upsert_position, set_completed}`, `MeditationLookup::list_all`.
   - DTOs (application/src/models.rs or per-model files per convention): `ExportDocument { format_version, exported_at, profile: { display_name }, mood_entries: [], practice: { sessions, notes, playback } }`, `MoodRecord`, `SessionRecord`, `NoteRecord`, `PlaybackRecord`, `ImportReport`.
   - AC: workspace compiles; `check-boundaries.sh` passes; fmt/clippy clean.

2. **Application use cases** (deps: 1)
   - `export/mod.rs`: gather own records → `ExportDocument`; meditation references resolved to name+duration via `MeditationLookup` (FR-001..004).
   - `import/mod.rs` (+ validation file): parse-agnostic — receives DTOs, validates every record (score 1–5, parseable times, known version) BEFORE any write (FR-008); duplicate set per D4 (FR-007); meditation match per D5 (FR-009/010); writes via sinks; returns `ImportReport` counts (FR-011); never touches identity (FR-014).
   - `export.tests.rs` / `import.tests.rs`: round-trip on fakes (SC-001), double-import all-duplicates (SC-003), unmatched-meditation skip (SC-004 partial), invalid record rejects with zero writes (SC-005 partial).
   - AC: tests green; clippy clean.

3. **Presentation: routes + formats** (deps: 2)
   - Router with the three routes (D7); handlers build `ExportDocument` via application, serialize JSON (`serde_json`) / CSV (`csv` crate) with attachment headers; import handler accepts multipart, `read_field_capped` (size cap: 10 MB), parses JSON or CSV (sniff: first non-space char `{` → JSON, else CSV mood-only), maps to DTOs, calls application, returns `ImportReport`.
   - CSV writer/reader as separate file(s) per one-definition-per-file; `.tests.rs` siblings: CSV round-trip with commas/quotes/newlines/unicode (spec edge), unknown version 400, malformed 400 (SC-002, SC-005).
   - AC: router tests green (auth required; other-user isolation asserted — SC-006); fmt/clippy clean.

4. **Composition root adapters + wiring** (deps: 1)
   - `src/apps/web-api/src/portability.rs`: adapters implementing the ports over `SeaOrmMoodEntryRepository`, `SeaOrmSessionRepository`, `SeaOrmNoteRepository`, `SeaOrmPlaybackStateRepository`, `SeaOrmMeditationRepository`; construct `SharedDependencies`, merge router (`router.rs`).
   - AC: server boots; `/api/portability/export/json` returns 200 + attachment for a logged-in user; boundary script green.

5. **UI: settings Data section** (deps: 3)
   - `settings.svelte`: "Your data" section — Download JSON / Download CSV buttons (fetch → blob → anchor download), import file input + submit with result counts (created / duplicates skipped / practice linked / skipped), error surface for rejects. Functional-basic styling (M8 owns design).
   - AC: `bun run build` passes; manual click-through on live server (login → export → import same file → counts shown, second import shows all-duplicates).

6. **Gate + live round-trip QA** (deps: 4, 5)
   - Full gate: `cargo fmt --check`, clippy `-D warnings` workspace, `cargo test --workspace`, `bun run build`.
   - Live QA on `.data` instance: seed user with mood entries + sessions/notes/positions; export JSON+CSV; wipe user data (fresh instance or re-registered user); import JSON → history/statistics/timeline identical (SC-001); import CSV → same (SC-002); second import → zero created (SC-003); remove one meditation then import → that meditation's records reported skipped, rest restored (SC-004); malformed file → 400, nothing written (SC-005); second user cannot reach first user's export (SC-006).
   - AC: all six SC demonstrated with evidence captured in the run report.

## QA / Evidence

- Per-task `.tests.rs` suites; final gate commands listed in task 6; live QA transcript (curl + UI walkthrough) recorded in the execution report. Traceability table below maps every criterion.

## Traceability

| Criterion | Tasks / Evidence |
| --- | --- |
| FR-001..004 (export JSON/CSV, name+duration refs, empty sections) | 2, 3 (export use case + serializers, tests) |
| FR-005/006 (import JSON/CSV) | 2, 3 (import use case + parsers, tests) |
| FR-007 (duplicate skip) | 2 (D4 set, SC-003 test) |
| FR-008 (atomic reject) | 2 (validate-then-write, SC-005 tests) |
| FR-009/010 (match/skip practice) | 2 (D5, SC-004 tests) |
| FR-011 (counts) | 2, 5 (ImportReport + UI display) |
| FR-012 (view consistency) | 6 (live QA re-checks history/analytics/timeline) |
| FR-013/014 (privacy, identity untouched) | 3, 4 (auth + caller-scoped ports; SC-006 test) |
| SC-001..006 | 2, 3 (unit/router), 6 (live evidence) |
| Preserved P1–P3 | 6 (regression: mood/profile suites still green; no repo changes per D3) |

## Blockers

None — all decisions resolved above (D1–D8).
