---
status: approved
spec: specs/0004-user-management/SPEC.md
---

# Implementation Plan: User Management — Account Lifecycle (Extension)

## Goal

Deliver the lifecycle extension of spec 0004 (US3–US5, FR-026..FR-029, SC-006..SC-010): admin disable/re-enable, admin delete with cross-module data wipe, member self-delete. The first delivery (create + list, SC-001..SC-005) is complete and shipped; this plan covers only the extension.

## Context

- **Branch:** `feat/m4-user-lifecycle` (spec extension already committed: `664c3b3`).
- **Users table** (`identity/infrastructure/.../m20260908_000001_identity_create_schema.rs`): `id, email, password_hash, role, must_change_password, auth_version, password_expires_at, created_at` — **no disabled flag**. Identity owns exactly one migration today; every module owns exactly one.
- **User domain** (`identity/domain/src/user/user.rs`): private fields + `restore(...)` constructor with an `InvalidCredentialState` invariant (`must_change_password == password_expires_at.is_some()`).
- **Session invalidation:** `web-api/src/session.rs::resolve_actor` loads the user, compares `auth_version`, and on mismatch/unknown-user deletes the session and inserts no `Actor`. Inserting a disabled check here kills live sessions without touching `auth_version`.
- **Session storage:** `tower_sessions_sqlx_store::SqliteStore` on the shared SQLite pool (table `session`, JSON data blob; sessions have no user_id column). `state.session_store: Arc<SqliteStore>` is already in `AppState`.
- **Cross-module wipe targets** (no cross-module FKs by architecture rule; `user_id` is a plain column):
  - meditation: `meditation_playback_state` (PK user_id, meditation_id), `meditation_sessions`, `meditation_notes`
  - soundscape: `soundscape_mixes` (user_id) and `soundscape_mix_items` (via `mix_id`)
  - mood-tracker: `mood_tracker_entries`
  - identity: `login_attempts` (keyed by normalized email), the `users` row itself
- **Media layout:** `$MEDIOSS_DATA_DIR/media/{user_id}/{opaque_id}`; `MediaStore::delete_folder(&folder)` already exists (`media-store/src/storage/media_store.rs:227`). Today no module writes user-scoped keys (meditation audio lives in program folders, soundscape sounds in library keys), so the folder wipe is idempotent future-proofing; shared content is never under `media/{user_id}/`.
- **Composition pattern:** web-api is the composition root; the M6 pattern (`web-api/src/mood_tracker.rs` adapting meditation's port for mood-tracker) is the precedent for orchestration without cross-module Rust deps.
- **Test layout:** identity presentation `router.tests.rs` (fake repository), application command tests (fake repo in `test_support/`), infrastructure `repository.tests.rs` (in-memory SQLite); each module's infrastructure has analogous repository tests.
- **Conventions:** definitions never in `mod.rs` (separate files); commits English conventional with FR refs in body; specs/plans English; bun for JS; Turkish UI strings.

## Decisions

1. **Disabled flag, not auth_version bump.** Add `disabled BOOLEAN NOT NULL DEFAULT 0` to `users`. `resolve_actor` drops sessions of disabled users; login rejects with the generic 401 shape. `auth_version` stays a password-change mechanism only. Re-enable flips the flag; password survives.
2. **Amend identity's single migration, no second file.** The app is unreleased and dev data is disposable (user rule: "migration sayısını minimal tutalım"; one migration per module is the repo invariant). The column is added to `m20260908_000001_identity_create_schema.rs` directly. **Rollout cost:** existing dev databases must be re-initialized (`rm -rf $MEDIOSS_DATA_DIR` or manual `ALTER TABLE users ADD COLUMN disabled BOOLEAN NOT NULL DEFAULT 0`). Flagged in the PR description.
3. **Endpoint shapes** (spec Clarifications allow plan-level choice):
   - `PATCH /api/identity/users/{id}/status` with `{"disabled": bool}` → `204` (disable and re-enable share one endpoint)
   - `DELETE /api/identity/users/{id}` → `204` (admin delete)
   - `DELETE /api/identity/me` → `204` (self-delete)
4. **Self-disable always 400** (US3 literal); self-delete allowed for admins only when another active admin exists (last-admin guard covers both paths).
5. **Wipe orchestration lives in web-api** (composition root), as an ordered, idempotent sequence ending in the user row (FR-027): module purges → `login_attempts` purge → session purge → media folder wipe → identity `delete_user`. A failed mid-sequence retry converges: every purge is idempotent and the guard re-validates.
6. **Cross-module purge via ports:** each module's application exposes `delete_all_for_user(user_id)` as a repository-port method + command; identity never imports other modules; web-api calls each module's command. Mirrors the timeline-adapter precedent.
7. **Session purge:** best-effort `DELETE FROM session WHERE json_extract(data,'$.\"user-id\"') = ?` (verify exact data shape at implementation; SQLite bundled builds include JSON1) plus the guaranteed lazy path — `resolve_actor` already deletes sessions of unknown users, and expiry cleans the rest. If the JSON shape does not match, ship the lazy path only and record it as a deviation; the observable contract (SC-008: subsequent requests 401) holds either way.
8. **List DTO gains `disabled`**; admin UI shows status + enable/disable + delete-with-confirm.
9. **Self-delete surface:** a minimal `/settings` page (Turkish) with a danger zone (typed confirmation phrase, `DELETE /api/identity/me`, redirect to `/login`). Profile settings (display name, avatar) remain spec-0008 scope; this page is the seed for it.
10. **CSRF:** new mutation routes follow whatever identity's existing mutation routes do (today: SameSite=Strict cookie, no Origin check — unlike mood-tracker's stricter Origin guard). If an Origin guard is added to identity during implementation, it applies to all identity mutations uniformly, not just the new ones.

## Constitution Check

- **Module boundaries:** no cross-module Rust deps (Decision 6); boundary check script must stay green.
- **mod.rs rule:** new commands/queries/entities each in own file.
- **Migration minimalism:** honored by amending (Decision 2), consistent with the user's pre-release rule.
- **Spec language:** English (spec already updated); commit messages English.
- **UI language:** Turkish (consistent with existing pages).
- **No new dependencies.**

## Scope

**In:** domain flag, migration amend, repo/port extensions, 3 commands, 3 routes, module purge ports (meditation/soundscape/mood-tracker), web-api orchestration + media wipe + session purge, admin UI lifecycle controls, `/settings` danger zone, tests at every layer, live QA.

**Out:** role changes, pagination, instance user limit (descoped), soft-delete/grace period, email change (descoped), profile display-name/avatar (spec 0008), Origin-guard retrofit of existing routes (only if free while adding new ones).

## Tasks

### T1 — Domain: disabled flag on User

`user.rs`: add `disabled: bool`; `new_permanent`/`new_temporary` default `false`; `restore(...)` gains a `disabled` param (update the two existing restore call sites); `is_disabled()` accessor. Domain tests: restore round-trips the flag; invariant untouched.

**Accept:** `cargo test -p identity-domain` green; compile ripple limited to known call sites.

### T2 — Infrastructure: migration amend + repository methods

Amend `m20260908_000001` (add column, keep `down()` irreversible). `users/entity.rs` gains the column. `SeaOrmUserRepository`: map it; add `set_disabled(id, bool)`, `delete_user(id)` (returns bool), `count_active_admins()`, `delete_login_attempts(normalized_email)`. Infra tests (in-memory SQLite): set/list round-trip, delete removes row, count_active_admins with disabled admin, login_attempts purge.

**Accept:** `cargo test -p identity-infrastructure` green; fresh DB migrates; boundary/fmt/clippy clean.

### T3 — Application: three commands + port extensions

Port: `set_disabled`, `delete_user`, `count_active_admins`, `delete_login_attempts` (trait + fake). Commands (own files, per mod.rs rule):

- `set_user_status(actor, target_id, disabled)` — admin-only guard at route level; command enforces: target exists (404-class), self-disable → 400, disabling the last active admin → 400.
- `delete_user_as_admin(actor, target_id)` — last-active-admin target → 400; deletes login_attempts + user row.
- `delete_own_account(actor)` — last-admin self-delete → 400; same deletes for own id.
- `authenticate` — reject disabled users with the existing generic 401 path.

Command tests with fake repo: guards (self, last-admin, 404), happy paths, disabled login.

**Accept:** `cargo test -p identity-application` green.

### T4 — Module purge ports

Each module: port method `delete_all_for_user(user_id) -> Result<u64/(), RepoError>` on its repository trait, SeaORM impl, small command exposing it, repository test asserting every user-owned table for that user is empty afterwards and other users' rows survive. Soundscape deletes `soundscape_mix_items` via `mix_id IN (SELECT id FROM soundscape_mixes WHERE user_id = ?)` before the mixes.

**Accept:** `cargo test -p meditation-infrastructure -p soundscape-infrastructure -p mood-tracker-infrastructure` green.

### T5 — Presentation: routes + DTO + route tests

`identity/presentation/src/router.rs` (or a new `users_lifecycle.rs` handler file if cleaner): `PATCH /users/{id}/status`, `DELETE /users/{id}`, `DELETE /me`; admin guard for the first two; `me` requires any authenticated actor. List DTO gains `disabled`. Route tests (fake repo): 204 happy, 400 last-admin/self, 403 member, 401 anonymous, list shows disabled.

**Accept:** `cargo test -p identity-presentation` green.

### T6 — web-api: orchestration + session kill

- `resolve_actor`: `user.is_disabled()` → delete session, insert no Actor (unit-testable via session.tests.rs pattern).
- New `web-api/src/user_deletion.rs`: ordered sequence (Decision 5) — module purges → login_attempts → session purge (Decision 7) → `media_store.delete_folder(&user_id.to_string())` → `delete_user`. Idempotent throughout; each step's error aborts with 500 and a tracing line naming the step.
- Wire into the two delete handlers via a `UserLifecyclePorts` struct assembled in `state.rs`/`main.rs` (composition root).
- Test: session.tests.rs-style coverage for the disabled check; orchestration exercised in T9 live QA (unit-level: each port is a thin call already tested in T4/T3).

**Accept:** `cargo test -p web-api` (or workspace) green; compile clean.

### T7 — UI

`api/identity.ts`: `setUserStatus`, `deleteUser`, `deleteOwnAccount` + `disabled` on the user type. `admin.svelte`: status column, enable/disable button, delete with `confirm()` (match existing confirm pattern used in mood history), error surface in Turkish. New `pages/settings.astro` + `settings.svelte`: danger zone, typed phrase (`SİL` or the account email — implementer's choice, stated in PR), on 204 → `window.location.assign("/login")`. Link `/settings` from the existing page headers where the other cross-links live.

**Accept:** `bun run build` clean; static assets synced (`cp -r dist/. ../web-api/static/`).

### T8 — Gates + commits

`cargo fmt --all`; `nix-shell -p rustfmt clippy --run "cargo clippy --workspace --all-targets -- -D warnings"`; `cargo test --workspace`; UI build; boundary check (make test). Commits grouped: (1) `feat(identity): user disable and delete commands with lifecycle guards` (T1–T3), (2) `feat(modules): per-user data purge ports` (T4), (3) `feat(identity): lifecycle routes and admin UI controls` (T5, T7 admin part), (4) `feat(web-api): cross-module user deletion orchestration` (T6), (5) `feat(ui): account self-deletion settings page` (T7). FR refs in bodies. Commit only on user request per repo rule — the user's flow here grants it when the PR step arrives.

### T9 — Live QA (SC-006..SC-009)

Fresh `$MEDIOSS_DATA_DIR`, server on :3000: admin setup → create member with data (mood entries, meditation completion, saved mix) → disable member → member login 401 generic, member's live session next-request 401 → re-enable → login succeeds with old password → admin delete member → admin list without member, member's moods/progress/mixes gone from every surface, `media/{id}/` absent, `session` rows for user purged-or-lazy → self-delete from `/settings` → 204, redirect, session dead → last-admin self-disable/self-delete → 400.

**Accept:** every SC-006..SC-009 scenario observed on the real surface; PR test plan updated with results.

## DoD

- All gates green; SC-006..SC-010 evidenced (traceability below).
- No cross-module dependency introduced; boundary check green.
- One migration per module invariant holds (identity amended, not appended).
- Dev-DB reinit note in PR description.

## Traceability

| ID | Task(s) | Evidence |
| --- | --- | --- |
| FR-026 (disable/401 generic/session kill) | T1, T2, T3, T6 | command tests; resolve_actor test; QA SC-006 |
| FR-027 (cross-module wipe + media) | T4, T6 | per-module repo tests; QA SC-009 |
| FR-028 (last-admin guard) | T3, T5 | command + route tests (400); QA |
| FR-029 (self-delete flush) | T3, T5, T7 | route test 204→401; QA SC-008 |
| SC-006 | T3, T5, T9 | route tests; live disable/re-enable |
| SC-007 | T4, T5, T6, T9 | route tests; post-delete table/dir inspection |
| SC-008 | T5, T7, T9 | route tests; `/settings` flow |
| SC-009 | T9 | live QA log |
| SC-010 | T8 | gate output |
| US3/US4/US5 | T3–T7 | covered via FR/SC rows |

## Blockers

None user-owned. Decision 2 (migration amend → dev DB reinit) is called out for visibility; speak now if you prefer a second migration file instead.
