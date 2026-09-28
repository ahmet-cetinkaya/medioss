---
status: complete
spec: specs/0004-user-management/SPEC.md
---

# Implementation Plan: User Management (Admin UI + List)

## Goal

Deliver approved spec 0004: allow admin to add users and view account list via web UI. `POST /api/identity/users` (FR-013) already exists and is tested; this plan adds only (1) `GET /api/identity/users` admin-only list endpoint, and (2) user management section in admin page (form + list + one-time password display). No existing identity behavior changes (PRESERVED: FR-004/012/013/014/018).

## Context

- **Existing API:** `POST /identity/users` router.rs:55, `create_user` command → `insert_member` → `User::new_temporary` (member role, must-change-password, 24h temporary hash). 403/409/400 paths are covered by command + route tests.
- **UserRepository** port has no `list` — only `count`, `find_by_email/id`, `insert_initial_admin_if_uninitialized`, `insert_member`, `apply_credential_change`. New `list_users` query requires port extension.
- **UI:** Astro pages (`pages/admin.astro` → `components/admin.svelte`), role check `fetchMe()` at line 63 (`me.role !== "admin"` → no content). No identity client under `lib/api/`; `fetchMe`/`login` are in `api/meditation.ts` (technical debt, not in scope of this plan but new client goes to `lib/api/identity.ts`).
- **Test structure:** presentation layer `router.tests.rs` (fake repository route tests), infrastructure `repository.tests.rs` (in-memory SQLite), application layer query tests.
- Convention: plan spec in `PLAN.md` in spec folder; commits use English conventional, body contains FR references.

## Tasks

### T1 — Application: `list_users` query

`identity/application/src/queries/list_users.rs` (+ test file, `mod.rs` registration): `list_users(repository) -> Result<Vec<User>, ApplicationError>`; empty repo → `vec![]`. Add `async fn list(&self) -> Result<Vec<User>, RepositoryError>` to `UserRepository` port; `fake_repository.rs` and `infrastructure/persistence/repository.rs` (SQL: `SELECT ... ORDER BY email ASC`) implement it. No domain change (User already has all fields).

Acceptance: `cargo test -p identity-application` green; `queries/list_users.tests.rs` (empty → vec![], seeded → email asc sorted).

### T2 — Presentation: `GET /identity/users` (admin-only)

`identity/presentation/src/router.rs`: new route `GET /identity/users` → `AdminActor` extractor (existing `CurrentActor` + role check pattern — same style as `AdminActor` in soundscape router) + `list_users` call → `200` `[{id, email, role, must_change_password}]` (password hash never serialized; `created_at` already exists). Origin check: not needed for GET (existing GET routes don't apply origin check — consistent with login/me).

Acceptance: 3 tests in `router.tests.rs`: admin → 200 + list correctly sorted; member → 403; anonymous → 401. Existing `POST /identity/users` tests unaffected.

### T3 — Web UI: identity client + admin page user section

`lib/api/identity.ts` (new): `fetchUsers()`, `createUser(email, password)` — `request()` helper imported/re-exported from `api/meditation.ts` (helper itself not copied). `admin.svelte`: continue existing layout with "Users" card — form (email + temporary password inputs), 409/400 messages shown below form in same language (FR-023), successful creation shows temporary password once (FR-025a; copyable, disappears on page refresh), list (id, email, role, must-change indicator) via `fetchUsers()`, member role already blocked at line 63 (FR-024 already addressed — only validation remains).

Acceptance: `bun run build` clean; manual QA (T5).

### T4 — Gates + commit

`cargo fmt --all`; `nix-shell -p rustfmt clippy --run "cargo clippy --workspace --all-targets -- -D warnings"`; `cargo test --workspace`; `cd src/apps/web-ui && bun run build`. Single commit (single cohesive change): `feat(identity): admin user management UI and user listing` — body contains FR-022..FR-025a + PRESERVED references. Commit only if user requests (repo rule).

### T5 — Live QA (SC-001..SC-004a)

Fresh data dir + server under `/tmp`: admin setup → create user from admin form (SC-001: 201, in list) → new user logs in with temporary password → `must_change_password: true` (SC-004a) → change password → normal usage. Same email again (different case) → 409 message in form (SC-002). Member session on admin page → list not fetched (SC-003). Route tests already cover SC-004.

## DoD

- All gates green (fmt, clippy `-D warnings`, `cargo test --workspace`, UI build).
- Live QA scenarios SC-001..SC-004a pass.
- Evidence exists for every FR/SC in traceability table.

## Traceability

| ID | Task(s) | Evidence |
| --- | --- | --- |
| FR-022 | T3 | QA: form → 201 + in list; route test not in T2 but create route exists |
| FR-023 | T3 | QA: 409/400 shown in form, form preserved |
| FR-024 | T4 | Existing line 63 gate + QA member session |
| FR-025 | T1, T2 | Route tests (200/403/401) + QA |
| FR-025a | T3 | QA: password shown once |
| SC-001..005 | T4, T5 | Gates + live QA outputs |

## Risks / Rollback

- Port extension (`UserRepository::list`) is breaking but only two impls (fake + SQLite) exist; both updated in this plan. Low risk.
- `created_at` already exists in repository SELECT/serialize path (repository.rs:71); no schema change — no migration needed.
- Rollback: single commit revert. API backward compatible (only additions).
- Authentication/throttle/argon2 paths untouched — PRESERVED list.
