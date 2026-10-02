---
status: living
capability: user-management
plan: specs/0004-user-management/PLAN.md
supersedes:
---

# M4 — User Management

## Intent

An authenticated admin can add users and view the account list from a single screen, without touching the application anywhere else. The API surface (`POST /api/identity/users`) has existed since M1; this spec puts the admin web UI surface and the list endpoint under contract. In the single-instance app, a user's lifecycle starts with the administrator: no self-service registration (PRD), no email infrastructure (PRD) — the temporary password is shown once on the admin's screen and delivered through a personal channel.

The user's lifecycle also **ends** here: an admin can disable or delete an account, and a member can delete their own account. Deletion is immediate and irreversible — all database rows and all media files under `media/{user_id}/` are wiped (PRD: Account & Data Deletion). Extended 2026-09-30 from the gap analysis.

## User Stories

### US1 — Admin creates a user (Priority: P1)

**Solo test:** As an admin, open the admin page; enter an email and a temporary password and submit; the new user appears in the list, and the created account is asked to change its password on first login.

- **Given** an admin session and an unused email, **when** the user is created, **then** the request returns `201`, the account is created with the `member` role and the must-change-password flag, and the UI shows the one-time password once, only on this screen.
- **Given** an email that already exists (including case differences), **when** the user is created, **then** the UI shows the `409` error as an error message in the same language, and the form is not cleared.
- **Given** a temporary password shorter than 8 characters, **when** the form is submitted, **then** the API returns `400` and the UI shows the validation error under the field.
- **Given** a member session, **when** the admin page is opened, **then** the page shows no content to non-admins and admin actions are rejected with 403.

### US2 — Admin lists users (Priority: P2)

**Solo test:** As an admin, open the admin page; the account list is displayed with email, role, and status.

- **Given** an admin session, **when** the admin page opens, **then** current accounts (id, email, role, must-change-password status, creation time) are listed in alphabetical email order.
- **Given** a member or anonymous client, **when** the list endpoint is called, **then** the request is rejected with 403/401 respectively.

### US3 — Admin disables a user (Priority: P1)

**Solo test:** As an admin, disable a member from the admin page; the account can no longer log in, and its active sessions are terminated immediately.

- **Given** an admin session and a member account, **when** the account is disabled, **then** the API returns `204`, the list shows the account as disabled, and every active session of that account is invalidated (further requests get `401`).
- **Given** a disabled account, **when** its owner tries to log in, **then** the response is `401` with the same generic shape as unknown credentials (no information leak that the account is disabled).
- **Given** a disabled account, **when** it is re-enabled by an admin, **then** the account can log in again with its existing password.
- **Given** an admin session and the last remaining admin account (self), **when** the admin tries to disable themselves, **then** the request is rejected with `400` (an instance must always keep at least one active admin).

### US4 — Admin deletes a user (Priority: P1)

**Solo test:** As an admin, delete a disabled member from the admin page with an explicit confirmation; the account, all its data rows, and all its media files are gone.

- **Given** an admin session and a member account, **when** the account is deleted after an explicit confirmation dialog (typing or clicking through), **then** the API returns `204`, the account disappears from the list, and afterwards: zero rows referencing the user exist in any module table (sessions, moods, meditation progress, notes, soundscape mixes), and `media/{user_id}/` is empty/removed on disk.
- **Given** a deletion, **when** other users' data is inspected, **then** nothing changes — shared library content uploaded by admins and other users' private data are untouched.
- **Given** the only admin account (self), **when** the admin tries to delete themselves, **then** the request is rejected with `400`.
- **Given** an anonymous or member client, **when** the delete/disable endpoints are called, **then** `401`/`403` respectively.

### US5 — Member deletes their own account (Priority: P2)

**Solo test:** As a member, delete my own account from a settings surface with explicit confirmation; my data and media are wiped and my session ends.

- **Given** an authenticated member (or admin, subject to the last-admin guard), **when** the account is self-deleted after an explicit confirmation, **then** the API returns `204`, the session is destroyed, all rows and media of the account are wiped as in US4, and the client lands on the login page.
- **Given** the request lacks the confirmation token re-read (e.g. typed phrase) required by the UI, **when** the form is submitted, **then** the UI blocks the request; the API itself requires no extra body parameter beyond the DELETE verb.

## Requirements

### ADDED

- FR-001: WHEN an admin-session client submits the form on the admin page with an unused email and a temporary password of at least 8 characters THE SYSTEM SHALL create the account with `201` and add it to the list in the UI; the account is born with the member role and the must-change-password flag.
- FR-002: WHEN the creation request returns `409` (email conflict) or `400` (validation) THE SYSTEM SHALL keep the form inputs and show the message next to the form in the same language.
- FR-003: WHEN a member-role user tries to access the admin page THE SYSTEM SHALL show the page empty/inaccessible and reject it without making API calls (the API already returns 403; the UI hides without touching anything).
- FR-004: WHEN the admin page loads THE SYSTEM SHALL be able to fetch current users (id, email, role, must_change_password, creation time) in alphabetical email order; the list endpoint is open to admin access only.
- FR-005: WHEN the temporary-password display area is used THE SYSTEM SHALL display the password only once at creation time and make it inaccessible in later renders and page reloads; the password is not stored persistently on the admin's screen.
- FR-026: WHEN an admin disables a member account THE SYSTEM SHALL reject login attempts with `401` (generic shape), terminate that account's active sessions immediately, and persist a disabled flag that re-enable can clear.
- FR-027: WHEN an admin or the account owner issues a valid delete THE SYSTEM SHALL remove, in one transaction or an ordered sequence ending in the user row, every database row referencing that user across all module tables and wipe `media/{user_id}/` on disk, returning `204`.
- FR-028: WHEN the last active admin would be disabled or deleted THE SYSTEM SHALL reject the request with `400` and keep the account intact.
- FR-029: WHEN a self-delete completes THE SYSTEM SHALL flush the caller's session so further requests get `401`.

## Success Criteria

- SC-001: The admin creates a user from the admin page; request `201`, account in the list, the new user is forced to change their password on first login (live QA: admin adds → member logs in with the temporary password → password change is requested).
- SC-002: When the same email is submitted again (different case), the UI shows a conflict error, the form is preserved; `409` is verified at the API level.
- SC-003: With a member session, opening the admin page does not fetch the user list and the creation action gets `403`; proven by route tests.
- SC-004: Route-level tests: admin `POST /api/identity/users` → 201; member → 403; anonymous → 401; `GET /api/identity/users` → admin 200 (sorted) / member 403.
- SC-005: All gates (fmt, clippy `-D warnings`, `cargo test --workspace`, UI build) green; live verification completed.
- SC-006: Route-level tests: admin disable/re-enable endpoint → 204 member / 400 last-admin / 403 member-role caller / 401 anonymous; disabled login → 401 generic shape; disabled account's live session → next request 401.
- SC-007: Route-level tests: admin `DELETE /api/identity/users/{id}` → 204; after deletion, no row referencing the user remains in any module table and `media/{user_id}/` is gone; last-admin self-delete → 400.
- SC-008: Route-level tests: self-delete endpoint → 204, session flushed, subsequent requests 401; member-role caller can delete only itself (deleting another user → 403).
- SC-009: Live QA: disable a member → member login fails; re-enable → login succeeds with existing password; delete a member with data → user gone from admin list, its mood entries/meditation progress/mixes absent from every surface, media dir removed.
- SC-010: All gates green (fmt, clippy `-D warnings`, `cargo test --workspace`, UI build) for the lifecycle extension.

## Edge Cases

- The temporary password is shown to the admin only on screen; it does not reappear after a page reload.
- If the email is entered with case differences, API normalization triggers `409`; the UI message is shown in the same form for both 409 and 400.
- The admin page is not rendered at all for non-admin users; it is hidden via the `fetchMe()` role check.
- Empty/invalid-format email → browser form validation blocks before the request; API `400` fallback.
- If two admins try to create the same email simultaneously, DB uniqueness produces `409`; a conflict message is shown in the UI.
- Disabling a user who is mid-session: the very next request returns `401`; no graceful warning is promised.
- Deletion of a user with active sessions: sessions die with the account rows; in-flight requests may complete but subsequent ones get `401`.
- If `media/{user_id}/` is already missing on disk, deletion still succeeds (idempotent wipe); orphan directories under other users' ids are never touched.
- Concurrent disable+delete of the same account: whichever wins, the loser gets `404`/`409`-class error, never a partial wipe.
- Member A deletes their own account at the same moment an admin disables it: both end states satisfy "no rows remain"; the last-writer guard is the last-admin rule.

## Assumptions

- `POST /api/identity/users` is used with M1 semantics; password generation, display, and the must-change flow are in this spec's scope.
- The list endpoint (`GET /api/identity/users`) was added with this module; no pagination (single instance, few users), ordering by email asc.
- The UI language stays Turkish (consistent with existing pages).
- No migration needed; the users table has existed since M1.
- Role constants: `admin` and `member` — the UI uses the `role === "admin"` check (the existing `fetchMe` contract is preserved).
- Disabled accounts keep their email uniqueness: re-creating a user with the same email while disabled still returns `409`; re-enable is the path back, not re-creation.
- Cross-module wipe enumerates every table with a `user_id` column at implementation time (sessions, mood entries, meditation progress/notes, soundscape mixes); the spec requires the enumeration be complete, verified by the exit test.
- Self-deletion requires typing a confirmation phrase in the UI; the API contract stays a bare DELETE of the caller's own account.

## Out of Scope

- Password reset/OTP flow, email invitations, self-service registration (will not exist per the PRD).
- Role changes (operator CLI tasks stay in M1).
- Pagination/search for the user list.
- Instance-level user limit configuration (descoped 2026-09-30).
- Account deletion is NOT deferred to the CLI: it is a first-class admin UI + API action here. Operator CLI stays as the M1 recovery path.
- Soft-delete or retention: deletion is hard and immediate; no grace period, no undo.

## Clarifications

### Session 2026-09-30 (lifecycle extension)

- Lifecycle work (disable/re-enable, admin delete, self delete) joins this spec instead of a new spec 0007 — user decision: "Specs/0004-user-management'e dahil edebiliriz. Yeni spec'e gerek yok gibi."
- Endpoint shapes are working contracts; the plan may adjust verbs/paths (e.g. `PATCH .../status` vs `POST .../disable`) as long as the SC-006..SC-008 behavioral contract holds.

## Deviations

None.
