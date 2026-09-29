---
title: User Statistics Implementation Plan
spec: specs/0005-user-statistics/SPEC.md
status: complete
created: 2026-09-28
---

# Plan — Spec 0005 User Statistics

## Goal

Implement Spec 0005: every authenticated user's own meditation practice summary (total minutes, total days, current/longest streak) and calendar, across Today / Last 30 days / All time ranges, with day boundaries following the client timezone offset.

## Context (repository facts)

- Session source: `meditation_sessions` (UNIQUE(user_id, meditation_id, started_at)), written only by `record_completion` (idempotent). Columns: `started_at`/`finished_at` TEXT (RFC3339 UTC).
- `MeditationSession` port: `application/src/ports/session_repository.rs` — currently only `record_if_new` + `count_for`; no listing port.
- Clock port: `ports/media.rs` `trait Clock` (async now → DateTime<Utc>) — used to resolve "today" for day boundaries.
- Presentation: `meditation/presentation/src/` (router.rs, extractors.rs, handlers_*.rs, router.tests.rs) — existing pattern: CurrentActor-style extractor + handler + router + test.
- UI: Astro pages (`src/apps/web-ui/src/pages/*.astro`) + Svelte components; API client `lib/api/meditation.ts` (uses the shared `lib/api/request.ts` helper).
- ARCHITECTURE.md: single SQLite, module-owned tables, no cross-module FKs; statistics read the meditation module's own table — no new module required.

## Decisions

| # | Decision | Rationale |
| --- | --- | --- |
| D1 | Statistics query lives in the **meditation module** as an application query (`get_user_statistics`) | The table belongs to the module; cross-module reads are forbidden (Constitution). A separate "statistics" module is needless layering for one table. |
| D2 | New port: `SessionRepository::list_for_user(user_id) -> Vec<MeditationSession>`; **aggregations run in application code, in memory** | With offset-day logic in code rather than SQL it stays testable; per-user session counts are small. Offset-based day grouping in SQLite is fragile. |
| D3 | API: `GET /api/meditation/statistics?range=today\|last30\|all&tz_offset_minutes=<i16>` | Existing REST pattern; the offset comes from the client per FR-005/A5. `range`/`tz_offset_minutes` are validated (FR-007). |
| D4 | Response DTO: `{range, total_minutes, total_days, current_streak, longest_streak, calendar: [{date: "YYYY-MM-DD", minutes}]}`; streaks always all-time (FR-003) | One endpoint, one shape; the UI refetches the same shape on range change. |
| D5 | UI: new `pages/stats.astro` + `fetchStatistics` in `lib/api/meditation.ts` + `Statistics.svelte` component; entry link from the history page header | New surface; existing page layout untouched. Calendar visualization: month grid with meditated days highlighted (out of scope: color scheme details). |
| D6 | No migration | Read-only derivation; schema unchanged (pre-release minimal-migration principle). |
| D7 | Day = the date of (`started_at` shifted by the offset); minutes = floor((finish−start)/60s) summed | Spec A2/A3 plus the midnight-crossing edge case. |

### Constitution Check

- Module boundary: the table belongs to meditation → the query lives in meditation ✔ (no cross-module FK).
- Identity: reuse the meditation presentation's existing user extractor; no new auth surface (FR-001/FR-006 come from the existing pattern).
- Migrations: none ✔ (minimal-migration principle).
- UI language: Turkish (repo-wide convention).

## Scope

**In:** D2 port (fake + SQLite), D3/D4 endpoint + validation, D5 UI (summary cards + calendar + range selector), unit/route tests, gates, live QA. **Out:** Spec's Out of Scope (other users' statistics, breakdowns, custom ranges, soundscape, export) + performance indexing (row counts are small; no measured speed contract).

## Tasks

- **T1 — Port + query (application)** `SessionRepository::list_for_user` (trait + FakeRepository + SQLite impl). New `queries/get_user_statistics.rs`: inputs `user_id, range, tz_offset_minutes, now(clock)`; output the D4 DTO; day/streak/range math per D7 + FR-003/A3. _Acceptance:_ unit tests in `queries.tests.rs` reproducing SC-001, SC-002, SC-003, SC-006 exactly (midnight-crossing session, DST fixed-offset, zero-duration included).
- **T2 — Presentation** `handlers_statistics.rs` following the `handlers_playback.rs` pattern: extractor for the user, `range`/`tz` validation (invalid → 400 FR-007), Clock dependency added to the router state (reused if present), `GET /statistics` route. Response per the `response.rs` pattern. _Acceptance:_ route tests: anonymous 401 (SC-005a), invalid range/tz 400, two users' data isolated (SC-004), empty user gets zero + empty calendar 200 (SC-006).
- **T3 — UI** `fetchStatistics(range, tzOffsetMinutes)` client; `Statistics.svelte` (4 summary cards + range selector + month-grid calendar; offset sent as `-new Date().getTimezoneOffset()`); `stats.astro` page; link from the history page header. _Acceptance:_ `bun run build` clean; the page redirects anonymous users to login (existing pattern); switching ranges refetches data (visual SC-005b).
- **T4 — Gates** fmt + clippy `-D warnings` + `cargo test --workspace` + UI build. _Acceptance:_ all green in a single run.
- **T5 — Live QA (fresh DB)** Setup → user → record sessions (varied days/offsets) → verify summary+calendar across all three ranges; member-B isolation; invalid parameters 400. _Acceptance:_ SC-001..006 observed live; curl outputs kept as evidence.
- **T6 — Spec status + records** SPEC.md `status: living` (reaches main on merge), PLAN `status: complete`. _Acceptance:_ files updated, included in the commit.

## Traceability

| Requirement                   | Task(s)  | Evidence               |
| ----------------------------- | -------- | ---------------------- |
| FR-001 (own data only)        | T1,T2    | SC-004 route test + QA |
| FR-002 (four metrics)         | T1       | unit test + DTO        |
| FR-003 (streaks unfiltered)   | T1       | SC-003 test            |
| FR-004 (calendar day+minutes) | T1,T3    | unit test + QA screen  |
| FR-005 (local day boundary)   | T1       | offset tests           |
| FR-006 (401)                  | T2       | route test             |
| FR-007 (400 validation)       | T2       | route tests            |
| FR-008 (range definitions)    | T1       | SC-003 test            |
| SC-001..SC-006                | T1/T2/T5 | tests + live QA        |

## Final Verification

T4 gates + T5 live QA scenarios; the plan is complete when `cargo test --workspace` and `bun run build` are green in a single run.

## Blockers

None — every decision is settled by evidence; no user-owned approvals pending.
