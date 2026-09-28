---
status: living
capability: user-management
plan: specs/0004-user-management/PLAN.md
supersedes:
---

# M4 — User Management

## Intent

An authenticated admin can add users and view the account list from a single screen, without touching the application anywhere else. The API surface (`POST /api/identity/users`) has existed since M1; this spec puts the admin web UI surface and the list endpoint under contract. In the single-instance app, a user's lifecycle starts with the administrator: no self-service registration (PRD), no email infrastructure (PRD) — the temporary password is shown once on the admin's screen and delivered through a personal channel.

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

## Requirements

### ADDED

- FR-001: WHEN an admin-session client submits the form on the admin page with an unused email and a temporary password of at least 8 characters THE SYSTEM SHALL create the account with `201` and add it to the list in the UI; the account is born with the member role and the must-change-password flag.
- FR-002: WHEN the creation request returns `409` (email conflict) or `400` (validation) THE SYSTEM SHALL keep the form inputs and show the message next to the form in the same language.
- FR-003: WHEN a member-role user tries to access the admin page THE SYSTEM SHALL show the page empty/inaccessible and reject it without making API calls (the API already returns 403; the UI hides without touching anything).
- FR-004: WHEN the admin page loads THE SYSTEM SHALL be able to fetch current users (id, email, role, must_change_password, creation time) in alphabetical email order; the list endpoint is open to admin access only.
- FR-005: WHEN the temporary-password display area is used THE SYSTEM SHALL display the password only once at creation time and make it inaccessible in later renders and page reloads; the password is not stored persistently on the admin's screen.

## Success Criteria

- SC-001: The admin creates a user from the admin page; request `201`, account in the list, the new user is forced to change their password on first login (live QA: admin adds → member logs in with the temporary password → password change is requested).
- SC-002: When the same email is submitted again (different case), the UI shows a conflict error, the form is preserved; `409` is verified at the API level.
- SC-003: With a member session, opening the admin page does not fetch the user list and the creation action gets `403`; proven by route tests.
- SC-004: Route-level tests: admin `POST /api/identity/users` → 201; member → 403; anonymous → 401; `GET /api/identity/users` → admin 200 (sorted) / member 403.
- SC-005: All gates (fmt, clippy `-D warnings`, `cargo test --workspace`, UI build) green; live verification completed.

## Edge Cases

- The temporary password is shown to the admin only on screen; it does not reappear after a page reload.
- If the email is entered with case differences, API normalization triggers `409`; the UI message is shown in the same form for both 409 and 400.
- The admin page is not rendered at all for non-admin users; it is hidden via the `fetchMe()` role check.
- Empty/invalid-format email → browser form validation blocks before the request; API `400` fallback.
- If two admins try to create the same email simultaneously, DB uniqueness produces `409`; a conflict message is shown in the UI.

## Assumptions

- `POST /api/identity/users` is used with M1 semantics; password generation, display, and the must-change flow are in this spec's scope.
- The list endpoint (`GET /api/identity/users`) was added with this module; no pagination (single instance, few users), ordering by email asc.
- The UI language stays Turkish (consistent with existing pages).
- No migration needed; the users table has existed since M1.
- Role constants: `admin` and `member` — the UI uses the `role === "admin"` check (the existing `fetchMe` contract is preserved).

## Out of Scope

- Password reset/OTP flow, email invitations, self-service registration (will not exist per the PRD).
- Role changes, account deletion/deactivation (operator CLI tasks stay in M1).
- Pagination/search for the user list.

## Clarifications

None.

## Deviations

None.
