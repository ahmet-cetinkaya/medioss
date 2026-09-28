---
status: in_progress
spec: specs/0003-soundscape-module/SPEC.md
---

# Implementation Plan: M3 — Soundscape Module

## Goal

Deliver the approved M3 Soundscape capability: an admin-curated shared ambient-sound library, private per-user saved mixes, a browser mixer for up to three independent looping sounds, emergency stop, and a cancellable final-minute fade-out timer. The implementation must preserve MediOSS's privacy and module boundaries, reuse proven local media behavior without coupling the soundscape module to meditation, and prove one-hour three-sound playback on the real browser surface.

## Context

- The approved contract is `specs/0003-soundscape-module/SPEC.md`; it has no clarification markers and contains 24 functional requirements and 10 success criteria.
- Workspace membership already globs `src/modules/*/{domain,application,infrastructure,presentation}` in `Cargo.toml`; adding four `soundscape` crates requires no root member edit.
- The established module shape is `src/modules/meditation/{domain,application,infrastructure,presentation}`. Each module owns its domain, use cases, persistence, migrations, and HTTP surface; cross-module Rust dependencies are forbidden by `scripts/check-boundaries.sh`.
- One SQLite database is shared, but each module owns prefixed tables and its own migration ledger. Soundscape uses `soundscape_*` tables and `soundscape_migrations`; `user_id` is a plain kernel `UserId` value with no cross-module foreign key.
- `src/shared/media-store` is the only shared blob primitive. It stores validated opaque `folder/file` keys under `$MEDIOSS_DATA_DIR/media`, supports collision-safe put/delete/open, and exposes `MediaStream`. It must remain feature-neutral and must not depend on soundscape.
- Meditation's audio validation and conversion live inside `meditation-infrastructure` and therefore cannot be imported. Soundscape mirrors the tested behavior in its own infrastructure adapters: byte sniff MP3/WAV/Ogg, use ffmpeg for full decode and Ogg Opus conversion, and ffprobe the stored duration. Do not extract a shared abstraction during M3; that would expand scope and weaken module ownership.
- The current meditation HTTP implementation buffers stored blobs before responding. M3 must instead wire `MediaStream` through the soundscape blob port and stream full/ranged responses so a three-loop mixer does not multiply whole-file buffers. Range semantics copy the verified meditation contract: valid single ranges return 206, malformed ranges are ignored and return 200, and unsatisfiable valid ranges return 416.
- Browser mixing belongs to the web UI: one `HTMLAudioElement` per selected sound, each with `loop = true`, independent `volume`, and coordinated play/pause/stop state. Server state is required only for the shared library and saved mixes; live player state and timers remain browser-session-local.
- Existing frontend conventions are Astro pages containing Svelte 5 islands, `$state`/`$derived`, same-origin typed fetch clients, redirect to `/login` on missing session, shadcn-svelte controls, Tailwind responsive classes, and `bun` as package manager.
- Existing real-binary E2E tests live under `src/apps/web-api/tests/` and spawn `CARGO_BIN_EXE_medioss` with a temporary data directory and no new HTTP client dependency. Router tests use injected fakes. Frontend currently has build-only CI, so deterministic mixer logic must be separated into testable TypeScript and a real-browser QA driver must cover actual media behavior.
- LSP is unavailable because `rust-analyzer` is not installed. Execution must use compiler diagnostics (`cargo check`/clippy/tests) and direct file inspection; do not install tooling unless separately authorized.

## Decisions

1. **Module boundary:** create a new `soundscape` business module with domain, application, infrastructure, and presentation crates. It may depend on `kernel`; only infrastructure/web-api composition may depend on `media-store`.
2. **Shared library ownership:** admin-only mutation, authenticated read/playback. Sound records are instance-wide, not user-owned.
3. **Category persistence (user decision, amended 2026-09-27):** use a `soundscape_categories` table. The initial migration seeds `Nature`, `Mechanical`, and `Ambient` as ordinary categories so a fresh instance is useful immediately; admins may create, rename, and delete categories through M3 CRUD. Seed by stable numeric ids and make migration reruns idempotent. Seeded categories are not protected from rename or deletion once empty.
4. **Schema:**
   - `soundscape_categories(id, slug UNIQUE, name UNIQUE)`.
   - `soundscape_sounds(id, name, category_id, content_type, size_bytes, storage_key UNIQUE, duration_ms, created_at)`.
   - `soundscape_mixes(id, user_id, name, created_at, updated_at)`.
   - `soundscape_mix_items(id, mix_id, sound_id, volume, position, UNIQUE(mix_id,sound_id), UNIQUE(mix_id,position))`.
   - `volume` is an integer percentage `0..=100`, avoiding floating-point persistence ambiguity; API responses use the same integer.
   - `position` is `0..=2` and preserves visible mix order.
5. **Delete semantics:** deleting a sound deletes its mix-item references first, then its database row, then its blob. Empty mixes remain. The command is retry-safe: a missing database row is `NotFound`; a missing blob after successful database cleanup is treated as already deleted. Failed database cleanup leaves the sound and blob intact.
6. **Storage layout:** sound blobs use the feature-owned `soundscape/<slug>-<suffix>.opus` folder/file shape through the existing media-store adapter. Only opaque keys are persisted. All accepted input is stored as Ogg Opus with `audio/ogg` content type.
7. **Upload contract:** cap multipart audio at 100 MB while reading; propagate chunk errors as 400; validate non-empty name and an existing category; sniff bytes; fully decode/convert; probe positive duration; store blob; insert row. On any failure after storage, delete the orphan blob. Existing media-store `put` collision safety is reused.
8. **HTTP API:** mount under `/api/soundscape`:
   - `GET /categories`
   - `POST /categories` (admin JSON: `name`)
   - `PUT /categories/{id}` (admin JSON: `name`)
   - `DELETE /categories/{id}` (admin; empty categories only)
   - `GET /sounds`
   - `POST /sounds` (admin multipart: `name`, `category_id`, `file`)
   - `PUT /sounds/{id}` (admin JSON: `name`, `category_id`)
   - `DELETE /sounds/{id}` (admin)
   - `GET /sounds/{id}/audio` (authenticated, full/single-range streaming)
   - `GET /mixes`
   - `POST /mixes` (name + ordered items)
   - `GET /mixes/{id}`
   - `PUT /mixes/{id}` (replace name and complete ordered item set atomically)
   - `DELETE /mixes/{id}`. Ownership misses return generic 404 to avoid revealing another user's mix. All mutations enforce the configured same-origin check.
9. **Category rules:** category names are trimmed, non-empty, bounded like other display names, and unique case-insensitively after normalization. Renaming preserves category id and assignments. Deleting a category that any sound references returns conflict and changes nothing; admins must first move or delete those sounds. Concurrent duplicate creates/renames rely on a database uniqueness constraint and return conflict.
10. **Mix validation:** names are trimmed and bounded consistently with existing content names; 1–3 distinct existing sound ids are required; every volume must be `0..=100`; positions must be contiguous from zero. Create/update validates the whole request before one transaction replaces data. Loading a mix omits deleted items and returns an `unavailable_sounds` list; normal deletion cleanup should make this list empty, but the shape covers concurrent deletion and legacy damage.
11. **Mixer UX:** add `/soundscape` with a library grouped by current categories, selected-sound cards, master play/pause, emergency stop, saved-mix controls, and timer controls. Add a Soundscape entry to authenticated home navigation and admin sound/category management sections on `/admin` rather than a second admin page.
12. **Playback model:** maintain no more than three audio elements keyed by sound id. Adding/removing/reordering does not recreate unaffected elements. Start all selected elements from the current position only after an explicit gesture. Independent sliders write `element.volume`; master pause preserves positions; emergency stop pauses, resets positions to zero, cancels timer/fade, and restores configured volumes.
13. **Looping:** set native loop for each element. Browser QA must observe loop transitions and one-hour continuity; no promise of sample-accurate phase synchronization is introduced.
14. **Timer:** allow preset or numeric durations within 5–120 minutes. Store an absolute deadline from a monotonic clock and derive remaining time; do not decrement state with an accumulating interval. During the final 60 seconds, apply a linear gain multiplier to each configured per-sound volume. Cancel restores configured volumes; replacement cancels the old schedule before installing the new one; zero invokes the same stop/reset path as emergency stop without clearing the selection.
15. **Background truthfulness:** listen to media play/pause/ended/error and document visibility changes. UI state is derived from actual elements, not only button intent. Background/lock-screen continuation is best effort; no service worker or external player is added.
16. **Testing strategy:** domain/application/persistence/router tests cover validation, category CRUD and in-use deletion conflict, ownership, transactions, cleanup, range behavior, and errors. Real-binary E2E covers category CRUD, admin upload/list/range/delete, regular-user rejection, two-user mix isolation, and delete cleanup. Pure timer/gain/state logic gets deterministic Bun tests with an injected clock and fake media handles. Real-browser QA covers simultaneous playback, controls, category management, autoplay handling, responsive/accessibility checks, background transition, looping, and the one-hour endurance run.
17. **No new runtime dependency by default:** use existing Rust/workspace dependencies and browser APIs. If implementation proves a new dependency unavoidable, stop and obtain explicit approval before adding it.

## Scope

### In scope

- New `soundscape` module across all four architecture layers.
- Admin-managed category CRUD with Nature/Mechanical/Ambient initial seeds and in-use deletion protection.
- Admin sound upload/edit/delete with conversion and cleanup.
- Authenticated list and streaming with byte ranges.
- Private saved-mix CRUD with 1–3 sounds and integer volumes.
- `/soundscape` mixer UI, admin management UI, and home navigation.
- Independent volume, master play/pause, emergency stop, looping, timer/fade, actual-state synchronization.
- Automated backend/frontend logic tests, real-binary E2E, and real-browser/manual QA including one-hour playback.

### Out of scope

- Per-user/private uploads, more than three sounds, or mix sharing.
- Category ordering, nesting, icons, tags, artwork, or broad search/filter.
- Recording/generation/editing, crossfade, equalizer, spatial audio, normalization, speed controls, or sample-accurate phase sync.
- Persisting live positions, active timer, or playing state across refresh/device.
- Offline sound caching, service workers, external catalogs/storage/telemetry, or guaranteed platform background playback.
- Six-hour certification.
- Refactoring meditation's media ports/adapters into a shared audio crate.
- Category ordering, nesting, icons, tags, artwork, or broad search/filter.

## Tasks

### 1. Scaffold the soundscape module and core domain

**Files:**

- Create `src/modules/soundscape/domain/{Cargo.toml,src/lib.rs,src/sound.rs,src/mix.rs}`.
- Create `src/modules/soundscape/application/{Cargo.toml,src/lib.rs,src/errors.rs}`.
- Add initial module manifests for infrastructure and presentation so boundary checks can inspect the full graph.

**Changes:**

- Define `SoundCategory`, `Sound`, `Mix`, and ordered `MixItem` value shapes.
- Encode domain invariants for non-empty bounded names, category identity, volume `0..=100`, 1–3 distinct items, and contiguous positions.
- Keep actor/role checks in presentation/application; the domain carries no session or database concepts.

**Acceptance:**

- Domain tests reject blank/overlong names, duplicate sounds, invalid volume/count/position and accept all three seeded categories.
- `cargo check --workspace --all-targets` and `scripts/check-boundaries.sh` accept the new dependency graph.

### 2. Add module-owned schema, seeds, entities, and repositories

**Files:**

- Create `src/modules/soundscape/infrastructure/src/migrations/{mod.rs,m20260927_000001_soundscape_create_schema.rs,migrations.tests.rs}`.
- Create `src/modules/soundscape/infrastructure/src/persistence/{mod.rs,entities/*.rs,category_repository.rs,sound_repository.rs,mix_repository.rs,persistence.tests.rs}`.
- Create application ports under `src/modules/soundscape/application/src/ports/`.

**Changes:**

- Add the four tables and constraints from Decisions, plus indexes for category listing and `user_id, updated_at` mix listing.
- Seed the three initial category rows idempotently while keeping them ordinary mutable rows.
- Implement repository ports for category list/find/create/rename/delete, sound CRUD/list, and owner-scoped mix CRUD/detail.
- Category create/rename enforce normalized uniqueness; category delete succeeds only when no sound references it and reports conflict otherwise.
- Implement create/update mix as one database transaction; delete-sound reference cleanup and row deletion are one transaction.
- Never return another user's mix from owner-scoped methods.

**Acceptance:**

- Migration tests prove fresh schema, exact initial seeds, constraints, idempotent rerun, and isolated `soundscape_migrations` ledger.
- Persistence tests prove category create/rename/delete, case-insensitive duplicate rejection, assignment preservation on rename, in-use deletion conflict, concurrent duplicate conflict, ordering, ownership isolation, atomic mix replacement, delete cleanup preserving empty mixes, and rollback on injected/constraint failure.

### 3. Build sound upload, media validation, conversion, and storage adapter

**Files:**

- Create `src/modules/soundscape/application/src/ports/media.rs` and commands `create_sound.rs`, `update_sound.rs`, `delete_sound.rs`.
- Create `src/modules/soundscape/infrastructure/src/media/{mod.rs,validator.rs,converter.rs}` with focused tests.
- Create `src/apps/web-api/src/soundscape.rs` for the media-store adapter and composition wiring.

**Changes:**

- Mirror the proven MP3/WAV/Ogg sniff/decode/Opus conversion behavior without importing meditation.
- Expose a soundscape `BlobStore` port whose read side returns a named async stream plus size, not `Vec<u8>`.
- Store to the `soundscape` folder with collision-safe media-store put; compensate stored blobs on row-insert failure.
- Implement retry-safe delete sequencing from Decision 5.

**Acceptance:**

- Validator tests cover genuine supported headers, truncated input, and disguised garbage.
- Converter tests cover pass-through Ogg Opus, conversion, decode failure, unavailable ffmpeg, and positive duration.
- Application tests prove 100 MB cap, missing category rejection, no retained row/blob on every failure stage, and no lost live blob on database cleanup failure.

### 4. Implement category, application queries, and private mix use cases

**Files:**

- Create `src/modules/soundscape/application/src/commands/{create_category.rs,rename_category.rs,delete_category.rs,create_mix.rs,update_mix.rs,delete_mix.rs}`.
- Create `src/modules/soundscape/application/src/queries/{list_categories.rs,list_sounds.rs,list_mixes.rs,get_mix.rs}`.
- Create `src/modules/soundscape/application/src/test_support/mod.rs` and command/query tests.

**Changes:**

- Validate category names and map duplicate/in-use outcomes to stable application errors.
- Validate complete mix requests before repository calls.
- Preserve client order, integer volume, and owner scoping.
- Resolve current sound metadata when returning mix details; omit missing sounds and populate unavailable identifiers/names when known.
- Sort sounds by category then name/id, and mixes by `updated_at DESC, id DESC`.

**Acceptance:**

- Tests cover category create/rename/delete, duplicate names, seeded-category mutation, in-use deletion conflict, and regular-user authorization at the presentation boundary.
- Tests cover mix create/list/get/update/delete happy paths, duplicate ids, limits, invalid volumes, nonexistent sounds, two-user isolation, ordering, deleted-item degradation, and no autoplay-related state in persisted DTOs.

### 5. Expose authenticated soundscape HTTP routes and streamed ranges

**Files:**

- Create `src/modules/soundscape/presentation/{Cargo.toml,src/lib.rs,src/router.rs,src/extractors.rs,src/response.rs,src/handlers_sounds.rs,src/handlers_mixes.rs,src/router.tests.rs}`.
- Update `src/apps/web-api/src/{state.rs,router.rs,main.rs,lib.rs,soundscape.rs}` and `src/apps/web-api/Cargo.toml`.

**Changes:**

- Add dependencies, run soundscape migrations at startup, construct dependencies, add state, and merge the router.
- Implement the routes from Decision 8 with generic 404 ownership behavior, admin extractor, category conflict responses, and same-origin mutation guard.
- Read multipart incrementally with the declared/actual 100 MB cap and propagate chunk errors.
- Parse only one byte range; stream the requested byte window from `MediaStream` without buffering the entire blob. Set `Accept-Ranges`, `Content-Type`, `Content-Length`, and `Content-Range` correctly.
- Return stable JSON DTOs for categories, sounds, mixes, mix items, and unavailable items.

**Acceptance:**

- Router tests cover unauthenticated 401, regular-user admin-route 403, missing/mismatched Origin 403, category duplicate/in-use conflict 409, admin success, malformed input 400/415/413, generic ownership 404, valid 200/206, malformed Range 200, unsatisfiable Range 416, and mid-stream failure behavior.
- A range test requests one byte from a multi-megabyte fake stream and proves the handler does not read the remainder.

### 6. Add real-binary soundscape E2E coverage

**Files:**

- Create `src/apps/web-api/tests/soundscape_e2e.rs` and reuse/copy the established local server/client harness without introducing a dependency.

**Changes:**

- Start the actual binary over a temp data directory; bootstrap admin and a second user.
- Exercise admin category create → rename → upload/use → refused in-use delete → move/delete sound → successful category delete.
- Exercise admin upload → list/category metadata → full/range stream → edit → delete.
- Exercise regular-user mutation rejection and two-user saved-mix isolation.
- Confirm delete removes media, strips mix items, preserves mix rows, and all requests remain local.

**Acceptance:**

- The E2E exits green against `CARGO_BIN_EXE_medioss`, validates category CRUD and conflict behavior, stored Ogg output/duration, 206 headers/body, 401/403/404 boundaries, and leaves no sound blob after delete.

### 7. Add the typed frontend client and deterministic mixer engine

**Files:**

- Create `src/apps/web-ui/src/lib/api/soundscape.ts`.
- Create `src/apps/web-ui/src/lib/soundscape/mixer.ts` and `mixer.test.ts`.
- Update `src/apps/web-ui/package.json` only to add a `test` script using built-in `bun test`; add no package.

**Changes:**

- Add typed same-origin API methods for category CRUD, sounds, admin mutations, and mix CRUD.
- Implement a small framework-independent `MixerController` around injected media handles and monotonic clock/scheduler interfaces.
- Keep at most three keyed handles, preserve unaffected handles, synchronize actual events, manage configured volumes separately from fade gain, and use one cancellation token/generation for timer replacement.

**Acceptance:**

- Bun tests deterministically prove three-sound cap, deduplication, independent volume, handle preservation, play/pause truth, emergency reset, timer replacement/cancel, final-minute linear fade, exact stop-at-zero, and stale callback immunity without sleeps.

### 8. Build the user mixer page and navigation

**Files:**

- Create `src/apps/web-ui/src/pages/soundscape.astro`.
- Create `src/apps/web-ui/src/lib/components/soundscape.svelte` plus small single-use child components only if the main file becomes materially unclear.
- Update `src/apps/web-ui/src/lib/components/home.svelte` navigation.

**Changes:**

- Load auth, categories, sounds, and mixes; redirect unauthenticated users to login.
- Render library groups, empty/error/loading states, selected slots, accessible sliders, master play/pause, emergency stop, timer controls/countdown, and save/load/rename/delete mix controls.
- Loading a mix replaces selection without autoplay and reports omitted sounds.
- At 320px, stack controls without clipping; at desktop, use library/mixer columns.
- Use semantic labels, keyboard-operable native inputs/buttons, visible focus, live regions for errors/status, and non-color-only playback state.

**Acceptance:**

- `bun run build` and `bun test` pass.
- Manual keyboard traversal reaches every control in a logical order; each slider/button has an accessible name; emergency stop remains visible and reachable at mobile/desktop widths.

### 9. Add admin category and sound-library management

**Files:**

- Update `src/apps/web-ui/src/lib/components/admin.svelte` or add `admin-soundscape.svelte` composed from it.
- Reuse `soundscape.ts` client types/functions.

**Changes:**

- Add category create/rename/delete controls with duplicate and in-use conflict feedback; seeded categories use the same controls as later categories.
- Add category-select upload form with editable title prefill, upload progress/disabled state, and clear success/error feedback.
- List sounds with category/duration/size and edit/delete actions; confirm destructive delete and explain that it removes the sound from saved mixes.
- Keep meditation admin functions intact and visually separate the two capabilities.

**Acceptance:**

- Admin can create and rename categories, cannot delete one in use, can delete it after moving/removing its sounds, and sees refreshed choices/grouping.
- Admin can upload/edit/delete sounds and sees refreshed state; regular user is redirected/refused.
- Invalid, oversized, and failed uploads leave the form recoverable and do not show phantom rows.

### 10. Run full verification and real-browser QA

**Files/artifacts:**

- Add `docs/qa/soundscape.md` only if the repository keeps repeatable manual scripts during execution; otherwise record commands/evidence in the execution ledger.
- No production changes unless QA exposes a defect.

**Changes/checks:**

- Run project gates: `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `./scripts/check-boundaries.sh --self-test && ./scripts/check-boundaries.sh`; `bun install --frozen-lockfile`; `bun test`; `bun run build`; Docker smoke.
- Launch the real app with generated local test loops and drive setup/login/admin upload/mixer/mix/timer flows in a real browser at desktop and 320px.
- Inspect network traffic for same-instance-only requests and verify autoplay blocked state.
- Run three distinct loops for 60 minutes, monitor that exactly three media elements remain, sample application/browser memory through the final 30 minutes, adjust all volumes, background/foreground the page, and observe loop transitions.
- Treat unbounded upward memory trend, lost controls, silent stopped tracks, stale timer callbacks, or false playing state as failures to fix before completion.

**Acceptance:**

- Every command exits zero; if clippy is unavailable locally, CI clippy success is required before readiness.
- Real-browser evidence covers SC-003 through SC-010, with screenshots at mobile/desktop and timestamped endurance observations.
- Existing identity and meditation flows still pass their automated suites and a short smoke check.

## Dependencies

- Task 1 precedes all module work.
- Task 2 depends on Task 1.
- Tasks 3 and 4 depend on Tasks 1–2 and may proceed in parallel after ports and domain shapes stabilize.
- Task 5 depends on Tasks 2–4.
- Task 6 depends on Task 5.
- Task 7 may begin after Task 5's DTOs/routes are fixed; its pure mixer engine can start in parallel with Task 6.
- Task 8 depends on Task 7 and stable read/mix APIs from Task 5.
- Task 9 depends on Task 7 and admin APIs from Task 5; it may run in parallel with Task 8.
- Task 10 depends on Tasks 1–9 and is the completion gate.

**Safe parallel groups:**

- Group A after Task 2: Task 3 media pipeline and Task 4 mix/query use cases, with one owner per file subtree.
- Group B after Task 5: Task 6 E2E and Task 7 mixer engine/client.
- Group C after Task 7: Task 8 user UI and Task 9 admin UI, provided shared client edits are assigned to one owner.

## QA and Evidence

### Automated evidence

- Domain/application tests: invariant and failure-path proof, including compensation and atomicity.
- Migration/persistence tests: schema, seeds, category CRUD/conflicts, ownership, transaction rollback, ordering, and cleanup.
- Router tests: auth, role, origin, validation, privacy-preserving 404, streaming range correctness, bounded reads.
- Real-binary E2E: full admin and two-user HTTP lifecycle over SQLite, ffmpeg, filesystem, sessions, and real routes.
- Bun tests: deterministic mixer/timer state without timing sleeps.
- Build/boundary checks: architecture and frontend compilation.

### Real-surface scenarios

1. Fresh instance → setup admin → verify three seeded categories → create and rename a fourth category → upload one sound in each desired category → verify grouped library → verify in-use deletion conflict and later empty-category deletion.
2. Login as user → select three → attempt fourth → play → change each volume → pause/resume → emergency stop.
3. Save mix → reload page → load mix → verify no autoplay and exact volumes → rename/delete.
4. Admin deletes one sound from a saved mix → owner loads mix → remaining items appear and unavailable item is reported/preserved mix exists.
5. Start short timer with test-clock QA mode only in non-production test harness → observe countdown, fade, cancellation restoration, replacement, and final stop.
6. Autoplay blocked: reload and verify UI says stopped until explicit gesture.
7. Mobile 320px and desktop: inspect layout, keyboard traversal, focus visibility, labels, contrast, error/live status.
8. Background/foreground and screen-lock where available: verify actual-state UI, not guaranteed continuation.
9. One-hour three-loop run: record start/end, loop continuity observations, controls at intervals, media element count, and memory samples for the final 30 minutes.
10. Browser network log: all application requests target the MediOSS origin; no analytics/external media calls.

### Risk gates

- **Persistent data/schema:** verify migration idempotence, seed stability, category uniqueness/in-use deletion protection, rollback, and backup compatibility before merging.
- **Authorization/privacy:** independent two-user E2E; owner predicates live in repository queries, not presentation filtering.
- **Untrusted media:** magic sniff plus full ffmpeg decode; fixed argv and piped bytes; no filename enters a command.
- **Resource use:** streamed reads, three-player hard cap, element reuse, one-hour memory observation.
- **Destructive delete:** explicit admin confirmation, transaction-first reference cleanup, retry-safe blob deletion, E2E proof.
- **Browser timing:** monotonic absolute deadline and deterministic fake-clock tests; no fixed sleeps in automated tests.

### Rollback evidence

- Before implementation, record the pre-change database state and confirm the new migration only creates `soundscape_*` tables/ledger.
- The new API/router can be unmounted and frontend links removed without changing identity/meditation data; soundscape tables and `media/soundscape/` are independently owned.
- Do not add a down migration that destroys user mixes/media. If rollout must be reverted, retain soundscape tables/blobs and roll back the binary; document cleanup as an explicit operator action rather than automatic data loss.

## Final Verification

1. Confirm spec remains `approved` until execution starts; executor then changes it and this plan to `in_progress`.
2. Confirm every task acceptance criterion has evidence and no unrelated refactor/dependency entered the diff.
3. Run the full automated gate once over the final tree and retain exact exit results.
4. Run all ten real-surface scenarios, including the one-hour endurance gate, on the final build.
5. Inspect changed files and `git diff --check`; verify no generated media, database, build output, secrets, or QA credentials are tracked.
6. Run an independent risk review focused on auth/privacy, deletion atomicity, range streaming/resource use, timer races, and test effectiveness; fix all blocking findings and rerun affected gates.
7. Mark the plan complete only when FR/SC traceability below has passing evidence and no `[NEEDS CLARIFICATION]` or open blocker remains.

## Traceability

The approved spec contains no `PB-*` items, so the table covers every `FR-*` and `SC-*` identifier.

| Requirement | Tasks | Evidence |
| --- | --: | --- |
| FR-001 | 2, 3, 5, 9 | application/router role tests; admin/regular-user E2E |
| FR-002 | 1, 2, 3, 5 | validation/converter/upload tests; invalid/oversize E2E |
| FR-003 | 3 | failure-stage compensation tests; filesystem E2E inspection |
| FR-004 | 2, 4, 5, 8 | query/router tests; grouped-library browser scenario |
| FR-005 | 3, 5, 6 | 200/206/416 tests; real-binary range E2E |
| FR-006 | 2, 3, 4, 6, 9 | transaction/delete tests; saved-mix delete E2E |
| FR-007 | 2, 4, 5, 6, 9 | category authorization tests; admin/regular-user E2E |
| FR-008 | 1, 2, 4, 5, 9 | category validation/uniqueness tests; rename assignment E2E |
| FR-009 | 2, 4, 5, 6, 9 | in-use delete conflict tests; browser/admin E2E |
| FR-010 | 2, 4, 5, 6, 8, 9 | empty delete tests; refreshed grouping/upload-choice QA |
| FR-011 | 1, 7, 8 | domain and mixer cap/dedup tests; fourth-sound browser scenario |
| FR-012 | 1, 7, 8 | volume invariant/mixer tests; accessible slider QA |
| FR-013 | 7, 8, 10 | handle-preservation tests; browser independent-volume observation |
| FR-014 | 7, 8, 10 | mixer loop tests; loop-transition and endurance observations |
| FR-015 | 7, 8, 10 | emergency-stop/timer-cancel tests; browser scenario |
| FR-016 | 7, 8, 10 | media-error event tests; mid-play failure QA |
| FR-017 | 1, 2, 4, 5, 8 | domain/use-case/router tests; save-mix browser scenario |
| FR-018 | 2, 4, 5, 6, 8 | owner-scoped persistence/router tests; two-user E2E |
| FR-019 | 4, 7, 8 | query DTO and no-autoplay mixer tests; reload/load browser scenario |
| FR-020 | 2, 4, 5, 6, 8 | deleted-item query tests; delete-then-load E2E/browser scenario |
| FR-021 | 7, 8 | timer range/replacement tests; browser timer controls |
| FR-022 | 7, 8 | countdown/cancel tests; visible timer QA |
| FR-023 | 7, 8, 10 | fake-clock fade tests; accelerated and real-surface fade evidence |
| FR-024 | 7, 8, 10 | cancellation-volume restoration tests; browser QA |
| FR-025 | 7, 8, 10 | media-event state tests; autoplay/background/error QA |
| FR-026 | 7, 8, 10 | visibility/media-event tests; supported-platform background QA |
| FR-027 | 2, 3, 5, 7, 10 | local persistence/filesystem evidence; browser network inspection |
| FR-028 | 5, 6 | unauthenticated router and real-binary E2E |
| SC-001 | 3, 5, 6, 9 | real-binary upload/list/stream/delete + role E2E |
| SC-002 | 2, 4, 5, 6, 9 | category lifecycle, in-use conflict, and role E2E |
| SC-003 | 2, 4, 5, 6 | two-user persistence/router/E2E isolation |
| SC-004 | 7, 8, 10 | timestamped one-hour three-sound browser run |
| SC-005 | 7, 10 | element-count assertions; final-30-minute memory samples |
| SC-006 | 7, 8, 10 | independent-volume and no-restart tests/browser observation |
| SC-007 | 7, 8, 10 | injected-clock tests; accelerated browser timer QA |
| SC-008 | 2, 4, 7, 8 | exact DTO/controller tests; load-mix browser scenario |
| SC-009 | 3, 5, 7, 10 | browser network log and self-hosted filesystem/database evidence |
| SC-010 | 8, 10 | 320px and desktop screenshots/interaction checklist |
| SC-011 | 8, 9, 10 | keyboard/accessibility audit of mixer and category management |
