# Implementation Plan: M2 — Meditation Module

**Spec**: `specs/0002-meditation-module/SPEC.md` (status: approved 2026-09-26) **Date**: 2026-09-26 | **Status**: approved (user ran `/ac:run` 2026-09-26) | **Suggested branch**: `feat/m2-meditation-module`

## Goal

Deliver the admin-curated shared meditation library per spec 0002: admin-only upload with Opus conversion, shared library/program/category views for every authenticated user, Range streaming, download, per-user playback resume, completion toggle, meditation sessions, and practice notes — as a new `meditation` module following the module-first Clean Architecture conventions established by `identity`.

## Context

- Workspace: `src/kernel`, `src/modules/<module>/{domain,application,infrastructure,presentation}`, `src/shared/media-store`, `src/apps/{web-api,web-ui}`. Workspace members use wildcards — adding `src/modules/meditation/*` requires no root manifest edit.
- `media-store` (M0) provides `MediaStore::put/open/delete` with `StorageKey`/`StorageNamespace` (currently `Meditation` only). Reused for audio and cover images; a `ProgramCover` namespace variant is added when covers land (task 7).
- `identity` conventions to copy: layer crate names; `migrations/` with a distinct ledger table (`meditation_migrations`); a generic deps struct + `<module>_router()` free function; presentation-local state type; `CurrentActor` extractor producing `Actor { user_id, role }`; strict `medioss_origin` Origin check on the module's own router.
- DB: one SQLite file, module-owned tables prefixed `meditation_`, no cross-module FKs (`UserId` stored as a plain column).
- web-ui: Astro + shadcn-svelte under `src/apps/web-ui`, package manager **bun** (never npm); identity already has first-run setup and login pages there.
- CI green bar: `cargo fmt --check`, clippy with warnings-as-errors, `cargo test --workspace`, the dependency-boundary check from ARCHITECTURE.md, and the Astro build.
- Opus: no mature pure-Rust encoder; the boring correct route is an `ffmpeg` subprocess (see D2). `ffmpeg`/`ffprobe` become host prerequisites (task 13) and must be present in the CI image (task 12).

## Decisions

- **D1 — Module boundary**: one new `meditation` module with four layer crates (`meditation-domain` → import `meditation_domain`, etc.). Zero edits to identity code; the only edit outside the new directory is mounting the router in the web-api composition root. No cross-module Rust dependency: the web-ui composes identity + meditation APIs client-side.
- **D2 — Opus conversion via ffmpeg subprocess**: accept → write body to a size-capped temp file → `ffprobe` to detect codec → if already Opus-in-Ogg, copy as-is (FR-039); else `ffmpeg -i tmp -c:a libopus -b:a 128k -vbr on` to a second temp file → `media-store put`. No user-controlled string ever reaches the command line (generated temp paths only). `ffmpeg` missing at runtime → typed infrastructure error surfaced as `503` on upload with a logged operator-facing message; nothing is stored and no original is silently retained. Exact bitrate flag (128k stereo default) may be tuned in task 4 but stays transparent-quality.
- **D3 — 100MB cap**: Content-Length pre-check → `413` before the body is read (FR-002); a streamed byte counter enforces the cap mid-body for undeclared/understated lengths → abort connection, delete temp file, store nothing (FR-003). The cap applies to the uploaded original; the converted Opus is naturally smaller. Exactly 100MB is accepted.
- **D4 — Content validation, two gates**: leading-signature sniffing (ID3/0xFFEx for MP3, RIFF/WAVE for WAV, OggS for OGG-family) before accepting bytes into the temp file; then ffmpeg's full decode as the second gate — corrupt-mid-stream files fail the encode and are rejected (FR-004, SC-003) with nothing persisted. Zero-byte uploads fail the sniff.
- **D5 — Route surface** (identity naming precedent, no version prefix; these names are canonical and pinned by the E2E suite):
  - Admin-gated: `POST /meditation/audio` (multipart: file, name) · `DELETE /meditation/{id}` · `POST /meditation/programs` (multipart: title, description?, cover?) · `PUT /meditation/programs/{id}` (json: title, description) · `DELETE /meditation/programs/{id}` · `POST /meditation/programs/{id}/members` (json: meditation_id) · `DELETE /meditation/programs/{id}/members/{meditation_id}` · `PUT /meditation/programs/{id}/category` (json: category_id) · `POST /meditation/categories` (json: name) · `DELETE /meditation/categories/{id}`
  - Authenticated read: `GET /meditation/library` · `GET /meditation/categories` · `GET /meditation/programs` · `GET /meditation/programs/{id}` · `GET /meditation/programs/{id}/cover` · `GET /meditation/{id}` · `GET /meditation/{id}/audio` (Range) · `GET /meditation/{id}/download` · `GET /meditation/{id}/playback-state`
  - Authenticated per-user writes: `POST /meditation/{id}/position` (json: position_ms) · `POST /meditation/{id}/completion` (json: started_at, finished_at) · `PUT /meditation/{id}/completed` (json: completed) · `GET /meditation/{id}/notes` · `POST /meditation/{id}/notes` (json: text) · `PUT /meditation/{id}/notes/{note_id}` · `DELETE /meditation/{id}/notes/{note_id}` (last two author-scoped)
  - Program progress is computed server-side per requesting user inside program list/detail responses (no separate route).
- **D6 — Schema** (module-owned, `meditation_` prefix, no cross-module FKs):
  - `meditation_meditations` (id PK, name TEXT NOT NULL, content_type TEXT NOT NULL, size_bytes INTEGER NOT NULL, storage_key TEXT NOT NULL UNIQUE, created_at)
  - `meditation_programs` (id PK, title TEXT NOT NULL, description TEXT NULL, cover_storage_key TEXT NULL, category_id INTEGER NULL, created_at)
  - `meditation_categories` (id PK, name TEXT NOT NULL UNIQUE, created_at) — unique instance-wide (shared structure)
  - `meditation_program_memberships` (id PK, program_id NOT NULL, meditation_id NOT NULL, added_at NOT NULL) — order = added_at
  - `meditation_playback_state` (user_id TEXT NOT NULL, meditation_id TEXT NOT NULL, position_ms INTEGER NOT NULL DEFAULT 0, completed INTEGER NOT NULL DEFAULT 0, updated_at; PK (user_id, meditation_id))
  - `meditation_sessions` (id PK, user_id, meditation_id, started_at, finished_at; UNIQUE (user_id, meditation_id, started_at) — the idempotency key from D8)
  - Migration ledger `meditation_migrations`; migration naming/style copied from identity's.
- **D7 — Authorization**: admin gate = presentation-layer check `actor.role == Role::Admin` on the D5 admin routes (403 otherwise, FR-005); every route requires an authenticated Actor (401 otherwise, FR-006) through the same session middleware identity uses. Practice data (playback state, sessions, notes) is always scoped to the acting user in the query layer (FR-023); another user's note id → 404 (FR-037).
- **D8 — Completion idempotency**: `POST /meditation/{id}/completion` carries client `started_at`/`finished_at`; the UNIQUE (user_id, meditation_id, started_at) key makes a retry return the same acknowledgement without a second session row (FR-020/021). On first acceptance the user's `completed` flips true and their position resets to 0 (FR-019 + spec assumption). A position report after completion simply overwrites the position (new pass, spec edge).
- **D9 — Covers**: validated by magic bytes (PNG/JPEG/WebP) with a 5MB cap, stored via media-store under a new `ProgramCover` namespace, served to any authenticated user at `GET /meditation/programs/{id}/cover`; missing cover → 404 and the UI shows a placeholder. Program deletion removes the cover bytes.
- **D10 — Streaming**: axum `Body::from_stream` over the media-store reader; Range parsed per RFC — single range only (multi-range → full 200), suffix (`bytes=-500`) and open-ended (`bytes=500-`) supported, satisfiable → 206 + `Content-Range`, unsatisfiable → 416 + `Content-Range: bytes */total`, malformed → ignore → 200 full (FR-009–013). Reads never modify the file; concurrent ranges are safe.
- **D11 — web-ui**: library page (program cards grouped under category headings, uncategorized last, per-user progress, cover/placeholder); program detail (member list with completed indicator); admin upload page (Title Case prefill: strip extension, `_`/`-` → space, capitalize words; editable field); admin program/category management; player page (audio element, speed selector 0.5–2.0 in 0.25 steps, seek bar, resume from playback-state); position posts throttled to every 5s of playback plus pause/unload (`navigator.sendBeacon` for unload); completion post when `currentTime >= duration - 1s`; completed toggle; notes CRUD under the player; download anchor to the download route.
- **D12 — E2E battery** (`src/apps/web-api/tests`, real binary, CI): pins the D5 route names and covers SC-001/003/004/005/006/016/017 as listed in Traceability. CI workflow gains ffmpeg/ffprobe in the image if not already present.
- **D13 — Delete semantics**: admin meditation delete removes metadata + bytes + memberships + **all users'** notes on it (FR-014); program delete removes program + memberships + cover bytes only (FR-028); category delete leaves programs uncategorized (FR-032). Regular-user practice rows for a deleted meditation are removed with it (spec edge).

## Scope

**In**: all of spec 0002 — 40 FR, 17 SC, 3 PB: admin upload with Opus conversion, shared library, programs/categories/covers, streaming + Range, download, per-user position/completed/sessions/notes, web-ui pages + player, E2E battery, docs note, CI green.

**Out**: the spec's Out of Scope list — per-user uploads/libraries, member reordering, multiple categories per program, post-upload rename, search/filter/streaks/statistics/calendar (M7), sleep timer (M3), auto-play (M7), offline/PWA (M8), original-file retention, sharing/export, soundscape (M3), seed content (M5), account-deletion wipe (M6), server-side duration extraction.

## Tasks

Convention: a task is done when its acceptance criteria hold AND the green bar passes in one run — `cargo fmt --check`, clippy `-D warnings` on the workspace, `cargo test --workspace`, boundary check, plus `bun run build` in `src/apps/web-ui` whenever UI files changed.

1. **Scaffold the meditation module** — four crates (`domain`, `application`, `infrastructure`, `presentation`) with empty lib targets and a `meditation_router()` stub; register in workspace via the existing wildcards. _Depends: none._ AC: `cargo build --workspace` green; boundary check passes with the new crates present; no behavior.
2. **Domain + application core for meditations** — domain `Meditation` entity with invariants (non-empty name, positive size, content type); application ports (`MeditationRepository`, `AudioStore` wrapping media-store, `AudioValidator`, `OpusConverter`) and use-cases: upload (validate → cap-check → convert → store → persist), list library, get metadata, delete (rows + bytes + memberships + notes, per D13). _Depends: 1._ AC: unit tests for invariants; use-case tests with fakes covering validator/converter rejection paths leaving nothing persisted, and delete removing all four targets.
3. **Infrastructure: schema + repositories** — migration creating the six tables + `meditation_migrations` ledger; SeaORM entities/mappers; repository implementations; playback-state/sessions/notes/programs/categories repositories. _Depends: 2._ AC: migration up/down tested in identity's style (reverse migration refused, database byte-unchanged); repository round-trip tests against a temp SQLite file, including the UNIQUE (user_id, meditation_id, started_at) session key.
4. **ffmpeg port: validator + Opus converter** — `infrastructure` implements `AudioValidator` (signature sniff, D4) and `OpusConverter` (ffprobe passthrough detection + ffmpeg encode, D2); typed error for missing ffmpeg. _Depends: 2._ AC: unit tests with tiny fixtures — valid MP3/WAV/OGG accepted; random-bytes `fake.mp3` rejected; corrupt-mid-stream file rejected at the decode gate; already-Opus file passed through byte-for-byte; a generated sine-wave WAV converts to a smaller Ogg Opus; absent ffmpeg produces the typed error.
5. **Presentation: audio surface routes** — upload (admin, multipart, 413 pre-read), library/metadata, audio streaming with Range semantics (D10), download (attachment, name-derived filename), meditation delete (admin). _Depends: 3, 4._ AC: router tests with fakes — 401 unauthenticated, 403 regular user on admin routes, 404 unknown id, 413 declared oversize before body read, mid-stream cap abort leaves nothing stored, 415 fake bytes, 200/206 (incl. `bytes=0-0`, suffix, open-ended)/416/malformed→200, true content type, attachment disposition.
6. **Presentation: playback state, completion, toggle** — position post, playback-state get, completion (D8 idempotency), completed toggle. _Depends: 5._ AC: router tests — duplicate completion retry → same ack, one session row; completion sets completed=true and resets position; toggle flips state without touching sessions; A's state invisible to B.
7. **Presentation: programs, categories, covers** — program CRUD (multipart with optional cover), membership add/remove, category CRUD + assignment, cover route (D9); per-user progress computed in list/detail responses; `ProgramCover` namespace added to media-store. _Depends: 5._ AC: router tests — admin/403/404 paths; duplicate category name rejected; fake/oversize cover rejected (415/413) with program creation proceeding only when cover valid or omitted; membership order by added_at; program delete keeps meditations; category delete leaves programs uncategorized; progress equals the requesting user's completed share, no fraction for empty programs; cover served to any authenticated user.
8. **Presentation: notes** — create/list/edit/delete, author-scoped (D7). _Depends: 5._ AC: router tests — B's list for a meditation is empty when only A wrote notes; B's PUT/DELETE on A's note → 404; edits apply to exactly that note; newest-first ordering.
9. **Composition root wiring** — mount `/meditation` in web-api with real implementations; ffmpeg presence check at startup (warn) surfacing 503 on upload when missing. _Depends: 5–8._ AC: web-api builds and starts; health endpoint green; smoke test: upload without ffmpeg on PATH → typed 503, nothing stored.
10. **web-ui: library, program detail, admin pages** — per D11. _Depends: 9._ AC: `bun run build` green; SC-012 grouping and SC-013 prefill verified manually on a dev server.
11. **web-ui: player, notes, download** — per D11. _Depends: 10._ AC: build green; SC-007 and SC-010 QA checklist executed in a real browser against a dev server.
12. **E2E battery + CI** — real-binary tests covering SC-001, SC-003, SC-004, SC-005, SC-006, SC-016, SC-017 and pinning D5 route names; add ffmpeg/ffprobe to the CI image if missing. _Depends: 9 (parallel with 10–11; final gate after 11)._ AC: battery green in CI; full workspace green bar in one run (SC-008/SC-009).
13. **Docs: deployment prerequisite** — DEPLOYMENT.md gains ffmpeg/ffprobe as host prerequisites and the media namespace layout note under `$MEDIOSS_DATA_DIR`. _Depends: 4._ AC: DEPLOYMENT.md updated; no other docs changed.

## Traceability

| Criterion | Task(s) | Evidence |
| --- | --- | --- |
| FR-001 upload | 2, 4, 5 | use-case + router tests, E2E SC-001 |
| FR-002 413 pre-read | 5 | router test |
| FR-003 mid-stream cap | 5 | router test (understated length) |
| FR-004 byte validation | 2, 4 | validator tests, E2E SC-003 |
| FR-005 403 regular user | 5, 7 | router tests, E2E SC-004 |
| FR-006 401 | 5–8 | unauthenticated cases of every router test |
| FR-007 shared library | 5 | router test |
| FR-008 metadata by id | 5 | router test |
| FR-009 full stream 200 | 5 | router test |
| FR-010 satisfiable Range 206 | 5 | router tests (open-ended, suffix, 0-0) |
| FR-011 unsatisfiable 416 | 5 | router test |
| FR-012 malformed Range → 200 | 5 | router test |
| FR-013 unknown id 404 | 5 | router tests |
| FR-014 delete cascade | 2, 5 | use-case test, E2E SC-006 |
| FR-015 position persist | 6 | router test |
| FR-016 playback-state get | 6 | router test |
| FR-017 player controls | 11 | SC-010 manual QA |
| FR-018 resume | 6, 11 | router test, SC-007 browser QA |
| FR-019 completed reopen from 0 | 6, 11 | router test (reset on completion), SC-007 |
| FR-020 completion record | 6 | router test |
| FR-021 completion idempotent | 6 | retry test (same started_at) |
| FR-022 toggle | 6 | router test (session count unchanged) |
| FR-023 per-user scoping | 6, 8 | scoping tests, E2E SC-005 |
| FR-024 Title Case prefill | 10 | SC-013 manual |
| FR-025 program create | 7 | router test |
| FR-026 cover validation/serving | 7 | router tests (415/413/200) |
| FR-027 membership add/remove | 7 | router test (added_at order) |
| FR-028 program delete | 7 | router test (meditations retained) |
| FR-029 progress share | 7 | router test per-user share, empty no-fraction |
| FR-030 category create | 7 | router test (uniqueness) |
| FR-031 category assign | 7 | router test |
| FR-032 category delete | 7 | router test (programs uncategorized) |
| FR-033 grouped library view | 10 | SC-012 manual |
| FR-034 note create | 8 | router test |
| FR-035 own notes newest first | 8 | router test |
| FR-036 note edit/delete | 8 | router test |
| FR-037 other's note → 404 | 8 | router test |
| FR-038 Opus conversion | 4, 5 | converter tests, E2E SC-016 |
| FR-039 Opus passthrough | 4 | already-Opus fixture test |
| FR-040 download | 5 | router test, E2E SC-017 |
| PB-1 identity stays green | 12 | full suite includes identity E2E |
| PB-2 media-store guarantees | 2, 7, 9 | reuse without behavior change; boundary check |
| PB-3 401 path | 5–8 | same middleware as identity |
| SC-001 E2E stream/404/401 | 12 | E2E |
| SC-002 413 pre-read | 5 | router test |
| SC-003 fake.mp3 415 | 4, 12 | unit + E2E |
| SC-004 admin-only mgmt | 5, 7, 12 | router tests + E2E |
| SC-005 per-user isolation | 6, 8, 12 | scoping tests + E2E |
| SC-006 delete leaves nothing | 2, 12 | use-case + E2E byte check |
| SC-007 browser resume | 11 | manual QA |
| SC-008 one-run green | 12 | CI run |
| SC-009 workspace checks | 12 | same CI run (template's SC-008/SC-009 merged here — see spec) |
| SC-010 playback QA checklist | 11 | manual QA |
| SC-011 progress math | 7 | router test (two programs, shared completion) |
| SC-012 grouped cards | 10 | manual QA |
| SC-013 prefill | 10 | manual QA |
| SC-014 notes privacy | 8, 12 | router tests + E2E |
| SC-015 toggle invariant | 6 | router test |
| SC-016 WAV→Opus | 4, 12 | converter test + E2E |
| SC-017 download | 5, 12 | router test + E2E |

## QA / Evidence Expectations

- Every task's completion runs the green bar (fmt, clippy -D warnings, workspace tests, boundary check, bun build when UI touched) in one run.
- E2E battery against the real binary in CI pins the route names and the ownership/isolation behaviors.
- Browser QA checklist (SC-007, SC-010, SC-012, SC-013) executed on a dev server and recorded as the delivery artifact.
- Disk spot-checks: after a WAV upload only the Opus file exists; after a delete, zero bytes remain under the media area.

## Final Verification

1. Green bar in one run: `cargo fmt --check`, clippy `-D warnings`, `cargo test --workspace`, boundary check.
2. `cd src/apps/web-ui && bun run build` green.
3. E2E battery green against the real binary in CI (ffmpeg present in the CI image).
4. Browser QA checklist fully checked (SC-007, SC-010, SC-012, SC-013).
5. DEPLOYMENT.md lists ffmpeg/ffprobe prerequisites.

## Blockers

None. ffmpeg is a documented host prerequisite (D2, tasks 9/12/13), not a user decision.

## User-owned decisions pending

None — spec 0002 approved 2026-09-26; all defaults live in its Assumptions.
