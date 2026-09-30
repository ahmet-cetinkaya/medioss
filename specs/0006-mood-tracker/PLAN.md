---
status: in_progress
spec: specs/0006-mood-tracker/SPEC.md
---

# Mood Tracker Implementation Plan

## Goal

Deliver the approved Mood Tracker capability: private 1–5 mood entries with optional notes, editable mood history, range-based statistics and charts, and one owner-scoped chronological timeline containing mood entries, meditation notes, and completed meditation sessions. Preserve existing meditation recording, notes, statistics, authentication, and administrator privacy behavior.

## Context

### Repository facts

- MediOSS is a Rust/Axum modular monolith with business-first Clean Architecture layers and one Astro/Svelte frontend (`docs/ARCHITECTURE.md`).
- Workspace globs already include `src/modules/*/{domain,application,infrastructure,presentation}`; a new `mood-tracker` module needs no explicit workspace member entries (`Cargo.toml`).
- Each module owns its tables and migrations. The API composition root explicitly runs each module migrator and mounts each presentation router (`src/apps/web-api/src/main.rs`, `src/apps/web-api/src/router.rs`, `src/apps/web-api/src/state.rs`).
- Authentication is resolved once in the API middleware and consumed by presentation-layer actor extractors. Existing meditation handlers and tests establish the 401 and owner-scoping conventions.
- Meditation sessions are owner-scoped through `SessionRepository::list_for_user`; meditation notes currently support listing by meditation only through `NoteRepository::list_for` and must gain an owner-wide read for the unified timeline.
- Meditation metadata is available through `MeditationRepository::find_by_id` and `list_all`; timeline assembly needs names for meditation-session and meditation-note context without crossing database ownership boundaries.
- Existing frontend conventions are Astro pages containing Svelte 5 islands, a shared typed `request` helper, shadcn-style UI components, and browser-derived timezone offsets (`src/apps/web-ui/src/lib/api/meditation.ts`, `statistics.svelte`).
- No chart package is installed. The required distribution and trend charts can use semantic HTML and native SVG; adding a dependency is unnecessary.
- Project gates are `make test` (format check, clippy with warnings denied, workspace tests, dependency boundaries), `make ui`, and `make build` (`Makefile`).
- The current spec is approved and contains no `[NEEDS CLARIFICATION]` markers.

### Constraints

- All personal mood and journal content is owner-only; administrator role grants no access.
- Domain and application crates may depend only on their own lower layer, `kernel`, and explicitly allowed framework-neutral libraries; `scripts/check-boundaries.sh` must remain green.
- The application is not yet published, so the new module should begin with one consolidated schema migration rather than incremental migrations.
- The plan does not authorize commits, pushes, destructive data operations, or deployment.

## Decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| D1 | Create a dedicated `mood-tracker` module with domain, application, infrastructure, and presentation crates. | Mood entries, lifecycle, analytics, and privacy form an independent business capability with owned persistence and HTTP behavior. |
| D2 | Keep the unified timeline query in the `mood-tracker` application layer, using explicit read ports implemented by composition-root adapters over meditation application ports. | The timeline is part of the Mood Tracker user outcome, while the mood module must not read meditation tables or depend on another module directly. Composition-root adapters preserve module ownership and Clean Architecture. |
| D3 | Extend meditation's application ports with owner-wide note listing and efficient meditation-name lookup needed by the adapter; implement them in fake and SeaORM repositories and cover them with meditation tests. | This is the smallest reusable API at the owner module boundary and avoids direct cross-module persistence access. |
| D4 | Model each mood entry with an opaque identifier, owner identifier, integer score, optional note, `recorded_at`, and `updated_at`; edits preserve `recorded_at`. | These fields directly support validation, privacy, chronology, and edit semantics without speculative metadata. |
| D5 | Use one initial Mood Tracker migration containing the owner index and timestamp/range indexes required by owner-scoped history and analytics. | The project is pre-release and explicitly prefers minimal migration count; one migration is sufficient for a new module. |
| D6 | Perform 1–5 validation, note trimming/10,000-character validation, range boundaries, daily averages, distribution percentages, and deterministic ordering in application use cases. | These are business rules and remain independently testable without HTTP or database coupling. |
| D7 | Use keyset pagination for history and timeline, ordered by `(recorded_at DESC, event-kind tie-breaker, id DESC)`, with opaque cursors and a bounded default page size. | It exposes every record without duplicates or omissions and provides stable ordering for equal timestamps without offset drift after mutations. |
| D8 | Define one range vocabulary (`today`, `last7`, `last30`, `all`) and one signed request-local timezone offset for mood history, analytics, and timeline endpoints. Reject unsupported ranges and invalid offsets with 400. | The approved behavior requires consistent range and local-day semantics across all views. |
| D9 | Add native SVG trend and semantic bar/distribution charts with accessible text/table equivalents; do not add a chart dependency. | The platform already supports the required output, keeps the bundle small, and permits exact WCAG-oriented labeling. |
| D10 | Add `/mood` for entry/history/analytics and replace the current `/history` content with the unified personal timeline while preserving the route and navigation affordance. | Users asked for one timeline; reusing the existing History destination avoids parallel histories and keeps existing links valid. |
| D11 | Treat mood deletion as a confirmation-gated permanent user action; test that all subsequent read models omit it. | This matches the approved spec and avoids adding an unrequested recovery lifecycle. |
| D12 | No telemetry, external calls, admin endpoints, correlation claims, reminders, tags, import/export, or offline synchronization are introduced. | These are explicitly outside the approved scope and conflict with privacy/YAGNI goals. |

### Constitution check

| Rule | Plan response |
| --- | --- |
| Business module first, Clean Architecture layers | D1 creates all four module layers and puts rules in domain/application, persistence in infrastructure, HTTP in presentation. |
| No cross-module business dependencies | D2 uses composition-root adapters; Mood Tracker crates do not depend on meditation crates. |
| Module-owned persistence | Mood Tracker owns only mood tables; meditation retains all reads of sessions, notes, and meditation metadata. |
| Thin shared kernel | Only `kernel::UserId` is shared; no new shared business abstraction is introduced. |
| Authentication at the edge, authorization in use cases | Presentation obtains the current actor; all repository operations and use cases require the owner ID. |
| Framework-free domain/application | Domain/application use existing neutral dependencies only; boundary checker proves compliance. |
| One deployable API and frontend | Composition changes stay in `web-api`; frontend remains the existing Astro/Svelte app. |
| Test and evidence discipline | Every task has direct tests and the final gate uses repository-defined commands plus real-browser QA. |

## Scope

### In scope

- Mood entry create, list, update, and confirmed delete.
- Score and note validation, multiple same-day entries, original timestamp preservation.
- Today / Last 7 days / Last 30 days / All time history and analytics.
- Entry count, arithmetic mean, score distribution, daily-average trend, and accessible charts.
- Unified timeline over mood entries, meditation notes, and completed meditation sessions.
- Owner isolation, administrator non-access, unauthenticated rejection, pagination, local-time grouping.
- UI routes, navigation, responsive/accessible states, API clients, module migration and composition wiring.

### Out of scope

- Emotion/activity tags, health diagnoses, crisis behavior, reminders, prompts, forecasts, sentiment or AI analysis.
- Correlation or causal claims between meditation and mood.
- Sharing or administrator inspection of personal data.
- Import/export, offline sync, soundscape events, changes to meditation recording/statistics/note semantics.
- New chart packages or generalized cross-module event infrastructure.

## Tasks

### T1 — Scaffold the Mood Tracker module and domain model

**Depends on:** none

Create the four crates under `src/modules/mood-tracker/`, following existing module manifests and public module structure. Define the `MoodEntry` domain entity and constrained score type; expose only fields required by the approved behavior. Add domain tests for valid endpoints 1 and 5 and rejection of missing-equivalent/out-of-range/non-integral values at application boundaries.

**Acceptance criteria**

- Workspace metadata discovers all four crates through existing globs.
- Domain/application crates contain no web, database, or runtime framework dependencies.
- Score construction cannot represent values outside 1–5.
- `cargo test -p mood-tracker-domain` passes.

### T2 — Define application ports, use cases, range semantics, and fakes

**Depends on:** T1

Define owner-scoped repository ports for mood CRUD and ranged pagination, a clock port, and read ports for meditation timeline events. Implement application commands/queries for create, update, delete, mood history, analytics, and unified timeline. Centralize note trimming and Unicode-character limit, supported ranges, fixed-offset local-day calculation, daily arithmetic means, exact distribution counts/percentages, no-data representation, cursor encoding/decoding, and deterministic merge ordering. Add in-memory fakes in application test support.

**Acceptance criteria**

- Application tests prove valid score-only and score-with-note creation, whitespace-to-absent notes, 10,000-character acceptance, over-limit rejection, multiple same-day records, and no mutation on invalid input.
- Owner mismatch is indistinguishable from an absent entry for read/update/delete paths.
- Edits preserve `recorded_at`; delete removes the entry from all subsequent application query results.
- Range tests cover local boundaries, just-inside/just-outside records, future-record exclusion, and offset-caused UTC/local date differences.
- Analytics tests prove SC-003 and SC-004 exactly, including no-data rather than score zero.
- Timeline tests merge all three event types, prove deterministic equal-timestamp ordering, and prove cursor traversal has no duplicates/omissions.

### T3 — Add the single Mood Tracker schema migration and repository

**Depends on:** T1, T2

Create one module migration with the mood-entry table, owner identifier, score constraint, nullable note, recorded/updated timestamps, and indexes matching owner/range/order queries. Implement the SeaORM repository with owner predicates embedded in every operation and keyset pagination. Add migration and repository integration tests using isolated SQLite databases.

**Acceptance criteria**

- A fresh database migrates up and down cleanly using the one Mood Tracker migration.
- SQLite rejects scores outside 1–5 as defense in depth.
- CRUD and pagination integration tests preserve owner isolation and deterministic order.
- Query tests cover empty datasets and data sets larger than one page.
- `cargo test -p mood-tracker-infrastructure` passes.

### T4 — Expose meditation-owned timeline reads

**Depends on:** none; safe in parallel with T1–T3

Extend `NoteRepository` with owner-wide newest-first note listing and implement it in the fake and SeaORM adapters. Use existing `SessionRepository::list_for_user` for completed sessions and existing meditation metadata reads for names; if batch lookup is needed to avoid repeated reads, add a minimal owner-neutral metadata lookup method to `MeditationRepository` rather than exposing persistence. Add meditation application/infrastructure tests for owner-wide note ordering and metadata assembly inputs.

**Acceptance criteria**

- Meditation notes can be listed across all meditations for exactly one owner, newest first.
- Existing per-meditation note behavior remains unchanged.
- Completed sessions remain sourced from existing idempotently recorded sessions.
- No Mood Tracker crate depends on a meditation crate.
- Existing meditation tests and boundary checks remain green.

### T5 — Wire cross-module timeline adapters in the composition root

**Depends on:** T2, T3, T4

Add Mood Tracker dependencies to `web-api`; implement composition-root adapters that satisfy Mood Tracker's meditation-read ports by calling meditation application ports. Construct Mood Tracker dependencies with the shared database connection and system clock, store them in `AppState`, run the Mood Tracker migrator at startup and in app test setup, and mount the Mood Tracker router under `/api`.

**Acceptance criteria**

- The API starts from a fresh data directory with all migrations applied.
- Composition adapters return only the requested owner's notes and completed sessions with meditation names.
- Mood Tracker module crates retain zero dependency on meditation module crates.
- Existing identity, meditation, and soundscape routes continue to mount and pass tests.

### T6 — Implement authenticated HTTP behavior and route tests

**Depends on:** T2, T5

Add handlers and routes for create/list/update/delete mood entries, mood analytics, and unified timeline. Reuse current actor extraction, browser-origin mutation protection, validation error conventions, and JSON response conventions. Accept range, timezone offset, page size, and cursor at the HTTP boundary; bound page size and map malformed inputs to 400. Require explicit delete confirmation in the UI rather than inventing a server-side confirmation token.

**Acceptance criteria**

- Presentation tests cover successful CRUD, score/note validation, ranges, analytics payloads, all timeline event variants, cursor continuation, and empty states.
- Anonymous requests to every mood/timeline endpoint return 401.
- Cross-user read/update/delete attempts disclose no mood contents; admin role gives no bypass.
- Unsupported ranges, invalid offsets/cursors/page sizes, and invalid scores return 400 without mutation.
- Mutation requests use the existing same-origin protection.
- `cargo test -p mood-tracker-presentation` passes.

### T7 — Add typed frontend API clients and Mood Tracker UI

**Depends on:** T6

Add a typed Mood Tracker API client using the shared request helper and browser timezone offset convention. Add `/mood` as an Astro page with a Svelte island that provides the 1–5 entry form, optional note, inline validation, entry history, edit/delete controls, range selection, summary cards, score-distribution chart, daily trend chart, loading/error/no-data states, and paginated continuation. Use native SVG and semantic text/table equivalents with labels that do not rely on color alone.

**Acceptance criteria**

- A user can create score-only and score-with-note entries, edit them without changing their displayed original time, and confirm deletion.
- The list and analytics update after each successful mutation without stale or contradictory values.
- Today / Last 7 / Last 30 / All time apply consistently.
- Charts expose equivalent values to assistive technology and remain readable at mobile and desktop widths.
- Empty, loading, validation, and API-failure states are actionable and do not fabricate data.
- Frontend type/build checks pass through the repository's UI command.

### T8 — Replace History with the unified timeline and update navigation

**Depends on:** T6, T7

Add timeline types/client calls and refactor the existing `/history` Svelte surface into the unified timeline. Render mood entries, meditation-note events, and completed-session events with distinct text labels/icons, local day grouping, local times, stable newest-first order, shared ranges, and cursor continuation. Preserve `/history` as the destination; add a clear Mood Tracker entry point from the home/history/statistics navigation without removing existing meditation/soundscape actions.

**Acceptance criteria**

- A six-record fixture matching SC-007 renders exactly six timeline events in correct order.
- Mood notes display with score; meditation notes display with meditation context; sessions display completion context.
- Local-date grouping proves SC-008 and does not reinterpret API calendar-date values through the browser's UTC conversion.
- Mood-only, meditation-only, and completely empty timelines render correctly.
- Navigation reaches both Mood Tracker and the unified History surface.

### T9 — Verify privacy, accessibility, responsive behavior, and regressions on the real surface

**Depends on:** T3, T6, T7, T8

Run scoped backend tests, frontend build, and the full repository gate. Start the application with isolated temporary data, create two normal members plus an administrator, seed or create representative mood/session/note records through supported interfaces, and drive the real browser at desktop and mobile widths. Capture API and visible-surface evidence for isolation, CRUD consistency, analytics math, chart accessibility, timeline ordering/pagination, local-date behavior, and empty states.

**Acceptance criteria**

- Every SC-001 through SC-012 has an observed test or real-surface artifact.
- Member A cannot retrieve Member B's data via UI or direct API; administrator gets no personal-data bypass.
- Browser QA covers happy path, invalid score, over-limit note, edit, confirmed delete, no-data, API error presentation, all ranges, and pagination.
- Keyboard navigation, focus visibility, accessible names, contrast, chart alternatives, and responsive layout satisfy the project's WCAG 2.1 AA target.
- `make test`, `make ui`, and `make build` exit 0 on the final tree.

## Dependencies and execution order

```text
T1 -> T2 -> T3 ----┐
T4 ----------------+-> T5 -> T6 -> T7 -> T8 -> T9
```

- T4 may run in parallel with T1–T3 because it changes only meditation-owned read ports and adapters.
- T3 begins after T2 fixes repository contracts and query requirements.
- T7 waits for stable HTTP contracts from T6.
- T8 reuses the timeline contract and UI primitives from T6/T7.
- T9 runs only over the integrated final state.

## QA and evidence

### Automated evidence

1. Domain/application unit tests for score/note rules, ownership, range math, analytics, ordering, cursors, and consistency.
2. SQLite migration/repository integration tests for constraints, CRUD, owner filtering, and pagination.
3. Presentation tests using the established Axum router test harness for auth, validation, privacy, payloads, and mutation origin checks.
4. Existing meditation tests after owner-wide note reads are added.
5. Frontend production build for TypeScript/Svelte/Astro correctness.
6. `make test` for formatting, clippy, workspace tests, and architecture boundaries.
7. `make build` for the deployable release artifact with embedded UI.

### Real-surface scenarios

- Create mood 1 without a note and mood 5 with multilingual/multiline text; both appear correctly.
- Reject 0, 6, a fractional score, a missing score, and a 10,001-character note without changing counts.
- Create scores 1, 3, 5 and observe count 3, average 3.0, and exact distribution.
- Create 2 and 4 on one local day and observe one daily trend point at 3.0.
- Edit an old record and confirm chronology is unchanged while all views update.
- Delete one record after confirmation and confirm disappearance from history, analytics, charts, and timeline.
- Traverse more than one page in history and timeline with no duplicate or missing IDs.
- View a timeline with all event types, equal timestamps, and UTC/local-date boundary fixtures.
- Log in as another member and administrator; direct API requests reveal no personal records.
- Inspect desktop and mobile layouts and keyboard/screen-reader semantics.

### Risk and rollback evidence

- Persistence risk is isolated to one additive Mood Tracker migration; verify its down migration on a disposable database before delivery.
- Cross-module risk is confined to read-only composition adapters and one owner-wide meditation note port; architecture boundary checks and meditation regression tests must pass.
- UI replacement risk is controlled by preserving `/history`, testing all three event types, and retaining navigation.
- No production or shared-state migration is authorized by this plan; execution uses local disposable data unless separately approved.

## Final verification

1. Re-read `specs/0006-mood-tracker/SPEC.md` and produce an FR/SC evidence ledger with no missing row.
2. Run `git diff --check` and inspect changed files for unrelated scope.
3. Run targeted crate tests for mood-tracker and affected meditation crates.
4. Run `make test` and require exit code 0.
5. Run `make ui` and `make build` and require exit code 0.
6. Start the final built application against isolated data and execute the real-surface scenarios at desktop and mobile widths.
7. Confirm no administrator or cross-user personal-data access and no external requests/telemetry.
8. Report residual risks and any pre-existing failures separately; do not declare ready with a missing required gate.

## Traceability

| Criterion | Tasks | Evidence |
| --- | --- | --- |
| FR-001 | T1, T2, T3, T6, T7 | Domain validation; create use-case/repository/route tests; browser create flow |
| FR-002 | T2, T3, T6, T7 | Optional-note create tests and UI flow |
| FR-003 | T1, T2, T6, T7 | Invalid-score unit/route/browser tests with unchanged count |
| FR-004 | T2, T6, T7 | Whitespace note tests and rendered absence |
| FR-005 | T2, T3, T6, T7 | Same-local-day multi-entry tests and history UI |
| FR-006 | T2, T3, T6, T7 | 1/10,000/10,001-character boundary tests |
| FR-007 | T2, T3, T6, T7 | Owner-scoped reverse-order repository/route/UI evidence |
| FR-008 | T6, T7 | History payload and full-note UI assertions |
| FR-009 | T2, T3, T6, T7 | Edit tests proving preserved `recorded_at` across views |
| FR-010 | T2, T3, T6, T7, T8 | Delete consistency tests and confirmed browser deletion |
| FR-011 | T2, T3, T6, T9 | Cross-user not-found/refusal and no content disclosure |
| FR-012 | T2, T3, T6 | Range-and-owner analytics tests |
| FR-013 | T2, T6, T7 | Exact count/mean/distribution payload and accessible charts |
| FR-014 | T2, T6, T7 | Same-day daily-average tests and trend point |
| FR-015 | T2, T6, T7 | Empty-range test and no-data UI |
| FR-016 | T2, T6, T7, T8 | Fixed-offset boundary tests and browser timezone fixtures |
| FR-017 | T2, T6, T7 | All four ranges in application/route/UI tests |
| FR-018 | T2, T4, T5, T6, T8 | Three-source merge tests and six-event browser fixture |
| FR-019 | T4, T5, T6, T8 | Meditation-note context/timestamp payload and timeline card |
| FR-020 | T2, T6, T8 | Mood score+note timeline assertions |
| FR-021 | T2, T6, T8 | Equal-timestamp ordering tests with deterministic tie-breaker |
| FR-022 | T2, T6, T8 | Shared-range timeline tests across all event types |
| FR-023 | T2, T3, T4, T5, T6, T9 | Owner filters at every source and direct API isolation checks |
| FR-024 | T5, T6, T9 | Admin direct API/UI non-access tests |
| FR-025 | T6, T9 | Anonymous route matrix returning 401 |
| FR-026 | T2, T6, T7, T8 | Post-mutation query tests and browser cross-view refresh |
| FR-027 | T2, T3, T6, T7, T8 | Cursor traversal tests and multi-page browser QA |
| SC-001 | T2, T6, T7, T9 | Score-only and score-with-note browser/API evidence |
| SC-002 | T1, T2, T6, T9 | Below/above/fractional/missing rejection with stable count |
| SC-003 | T2, T6, T7, T9 | Fixture 1/3/5 yields count 3, average 3.0, exact buckets |
| SC-004 | T2, T6, T7, T9 | Same-day 2/4 fixture yields one 3.0 trend point |
| SC-005 | T2, T3, T6, T7, T8, T9 | Edit consistency and preserved timestamp evidence |
| SC-006 | T2, T3, T6, T7, T8, T9 | Delete consistency across every view |
| SC-007 | T2, T4, T5, T6, T8, T9 | Two of each event type render as six ordered events |
| SC-008 | T2, T6, T8, T9 | UTC/local-date fixture grouped under one local date |
| SC-009 | T2, T3, T4, T5, T6, T9 | Member/admin isolation route and browser evidence |
| SC-010 | T6, T7, T8, T9 | First-user empty states without fabricated values |
| SC-011 | T7, T8, T9 | Desktop/mobile WCAG audit and accessible chart alternatives |
| SC-012 | T2, T3, T6, T7, T8, T9 | Over-page fixture traversed with no duplicates/omissions |
