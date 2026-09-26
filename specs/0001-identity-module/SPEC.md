---
status: draft
capability: identity
plan:
supersedes:
---

# M1 — Identity Module

## Intent

A single-instance wellness app must let exactly one operator bootstrap an admin account, let users authenticate, and let the admin manage accounts — with no external identity provider and no email infrastructure, so that all identity data stays on the host (PRD privacy trust). This change delivers the identity capability: first-run admin setup, login with exponential-backoff throttling, session lifecycle safe against fixation, self password change, admin-driven user creation, and host-side operator CLI for password reset and throttle clearing. The behavior is already implemented and under review on PR #2 (`feat/m1-identity-module`); this spec formalizes it as the contract the implementation is verified against.

## User Stories

### US1 — First-run admin setup (Priority: P1)

An operator can turn a fresh instance into a usable one by creating the first admin account, and can discover whether that step is still pending.

**Independent test:** on a fresh database, `GET /identity/setup-status` returns `setup_completed: false`; a setup request creates the admin; a second setup request is refused; the status now reports completed.

- Given a fresh instance, when the operator submits a valid email and password to setup, then exactly one admin account exists and subsequent setup attempts are refused with `409 setup already completed`.
- Given two setup requests race concurrently on a fresh instance, when both complete, then exactly one admin row exists and only one request succeeds.
- Given any client, when setup status is queried, then no authentication is required.

### US2 — Login with throttling (Priority: P1)

A user with an account can log in with email and password; repeated failures slow an attacker down without permanently locking the account, and without revealing whether the email exists.

**Independent test:** submit wrong credentials repeatedly until responses turn `429` with a wait hint; submit correct credentials during the block and observe refusal; clear the throttle from the host and observe a successful login.

- Given valid credentials, when a login is submitted, then the session id is rotated before any authentication state is written into the session, the throttle record for the identifier is cleared, and the actor is bound to the session.
- Given unknown email or wrong password, when a login is submitted, then the response is `401` with the same generic message and shape in both cases, and the failure is recorded against the submitted identifier.
- Given `n` consecutive failures for one identifier, when the next attempt is evaluated, then the required wait is `2^(n-1)` seconds capped at 64 seconds.
- Given the identifier is inside its block window, when a login is submitted, then it is refused with `429` carrying the remaining seconds in the body and a `Retry-After` header of at least 1 second — even if the credentials are correct.
- Given the block window has passed, when a login is submitted, then throttling does not count the expired block against the attempt.

### US3 — Authenticated session lifecycle (Priority: P1)

A logged-in user stays logged in across requests, can see who they are, and can log out; broken session storage must never silently log users out or leave fixation windows open.

**Independent test:** log in, call the identity `me` endpoint and observe the actor; log out and observe the session no longer authenticates; simulate a store failure and observe `500` instead of a logged-out actor.

- Given an authenticated session, when the actor endpoint is called, then the current user's id, email, and role are returned.
- Given an authenticated session, when logout is called, then the session is fully flushed and `204` is returned.
- Given the session store fails, when any authenticated request is served, then the server returns `500` and logs the cause — the actor is never silently treated as logged out.
- Given a session id that is structurally valid but unknown, when any request is served, then the request proceeds as unauthenticated (cache miss, not an error).
- Given a session cookie, when it is issued, then it is host-locked, http-only, same-site lax, secure unless explicitly disabled for local development, and expires per the configured timeout (default 7 days).

### US4 — Change own password (Priority: P2)

A logged-in user can replace their own password without operator help.

**Independent test:** change the password with a valid new one, log out, log back in with the new password; repeat with an invalid password and observe a validation error naming the rule.

- Given an authenticated session and a password satisfying the policy, when the change is submitted, then the stored credential is replaced, the session id is rotated, and the actor stays logged in.
- Given a password shorter than 8 characters, when the change is submitted, then the response is `400` with the domain message stating the minimum length — not a generic error string.
- Given a temporary-password user who has not changed their password yet, when they authenticate, then the account still requires the password change flow before normal use.

### US5 — Admin creates users (Priority: P2)

An admin can grant access to another person by creating their account; only admins can do this.

**Independent test:** as admin, create a user with a fresh email and observe success; repeat with the same email (different case) and observe refusal; as a non-admin, observe `403`.

- Given an admin session and an unused email, when a user is created, then the account exists with role `user` and the response echoes id, email, and role.
- Given an email that already exists in any letter case, when a user is created, then the response is `409 email already exists`.
- Given a non-admin session, when a user creation is attempted, then the response is `403 forbidden`.

### US6 — Operator password reset (Priority: P2)

A host operator can restore access to a locked-out account without a running server and without email infrastructure, by issuing a temporary password.

**Independent test:** run the reset subcommand for an existing email, observe a readable temporary password on stdout, then log in with it.

- Given an existing account, when the reset subcommand runs, then a temporary password (20 characters drawn uniformly from a confusable-free alphabet) is printed to the operator, the stored credential is replaced with an expiring hash (24 hours), and the account is marked as must-change-password.
- Given the reset subcommand hits any failure, when it exits, then the full error cause chain is printed and the exit code is non-zero.
- Given a nonexistent email, when the reset subcommand runs, then it prints `no such user` and exits non-zero without creating anything.

### US7 — Operator throttle clearing (Priority: P3)

A host operator can clear a login throttle for an account that was blocked (e.g. after a shared-IP incident) without touching the database by hand.

**Independent test:** block an account by repeated failures, run the clear-throttle subcommand, and observe the next login behave as if no failures were recorded.

- Given a throttled identifier, when the clear-throttle subcommand runs, then the recorded failure state for that identifier is removed and login attempts start from a clean slate.
- Given a nonexistent identifier, when the clear-throttle subcommand runs, then the command completes without error (nothing to clear).

### US8 — Credential storage hardening (Priority: P1)

Password verifiers must be worthless to an attacker who steals the database.

**Independent test:** verify that stored verifiers embed their cost parameters, that upgrading parameters changes newly stored verifiers while old ones still verify, and that unknown-email and wrong-password logins take equivalent time (unit-verifiable via the dummy-verify path).

- Given a password is stored, when it is persisted, then only a memory-hard hash with its parameters embedded is stored — never the password.
- Given new credentials are written, when hashing happens, then cost parameters are used that exceed the verification budget (new hashes are harder than the minimum a stored old hash requires).
- Given a login attempts an unknown email, when the response is produced, then a fixed dummy verification runs so the response time matches the known-email path, and the final comparison is constant-time.

## Requirements

### ADDED

- FR-001: WHEN setup status is queried on any instance THE SYSTEM SHALL report whether an admin account exists, without requiring authentication.
- FR-002: WHEN setup is submitted with a valid email and password while no admin exists THE SYSTEM SHALL create exactly one admin account and refuse every later setup attempt with `409 setup already completed`.
- FR-003: WHEN setup requests race concurrently on a fresh instance THE SYSTEM SHALL persist exactly one admin (the setup claim is atomic and idempotent across restarts).
- FR-004: WHEN a login presents valid credentials THE SYSTEM SHALL rotate the session id before writing authentication state into the session, bind the actor, and clear the identifier's failure record.
- FR-005: WHEN a login presents an unknown email or a wrong password THE SYSTEM SHALL respond `401` with one shared generic message, indistinguishable between the two cases.
- FR-006: WHEN a login fails THE SYSTEM SHALL record the failure against the submitted (normalized) identifier and require a wait of `2^(n-1)` seconds after `n` consecutive failures, capped at 64 seconds.
- FR-007: WHILE an identifier is inside its block window THE SYSTEM SHALL refuse login with `429`, the remaining seconds in the body, and a `Retry-After` header of `max(seconds, 1)`, regardless of credential validity.
- FR-008: WHEN the block window has expired THE SYSTEM SHALL evaluate the next attempt as if no active block existed.
- FR-009: WHEN the session store fails during a request THE SYSTEM SHALL return `500` and log the cause; an unknown-but-valid session id SHALL be treated as unauthenticated, not as an error.
- FR-010: WHEN logout is called by an authenticated actor THE SYSTEM SHALL flush the session, return `204`, and propagate store failures as errors.
- FR-011: WHEN an authenticated user submits a password change with a password of at least 8 characters THE SYSTEM SHALL replace the stored verifier, rotate the session id, and keep the user logged in.
- FR-012: IF a submitted password is shorter than 8 characters THEN THE SYSTEM SHALL reject it with `400` and the domain message naming the rule.
- FR-013: WHEN an admin creates a user with an unused email THE SYSTEM SHALL create the account with role `user`; a duplicate email in any letter case yields `409`; a non-admin caller yields `403`.
- FR-014: WHEN an email is stored THE SYSTEM SHALL normalize it to lowercase and enforce uniqueness case-insensitively at the database level.
- FR-015: WHEN the reset-password subcommand runs for an existing email THE SYSTEM SHALL replace the credential with a hash expiring after 24 hours, print a uniformly sampled temporary password (minimum 8 characters) to the operator, and mark the account must-change-password.
- FR-016: WHEN the clear-throttle subcommand runs for an identifier THE SYSTEM SHALL remove that identifier's failure record and succeed even if none existed.
- FR-017: WHEN the reverse migration of the identity schema is invoked THE SYSTEM SHALL fail with an explanatory error and destroy no data; rollback is restore-from-backup only.
- FR-018: WHEN a password verifier is stored or checked THE SYSTEM SHALL use a memory-hard hash whose parameters travel inside the stored string; verification SHALL honor embedded parameters, new verifiers SHALL use cost parameters above the verification budget, comparison SHALL be constant-time, and unknown-email logins SHALL run a fixed dummy verification to equalize timing.
- FR-019: WHEN a session cookie is issued THE SYSTEM SHALL set host-locked, http-only, same-site lax attributes, the secure attribute unless explicitly disabled for development, and an expiry from the configured timeout.
- FR-020: WHEN CLI subcommands fail THE SYSTEM SHALL print the full error cause chain and exit non-zero.
- FR-021: IF a stored role value is not one of the known roles THEN THE SYSTEM SHALL surface a typed repository error instead of panicking or guessing a default.

### PRESERVED

- PB-1: Session ids issued before authentication are never reused after it; every login and own-password change rotates the session id before auth state is written.
- PB-2: The existence of an account is never disclosed through login responses: unknown email and wrong password share one generic `401`.

## Success Criteria

- SC-001: On a fresh instance, the E2E flow setup-status → setup → setup-status shows `false → created → true`, and a second setup request returns `409`.
- SC-002: Concurrent setup claims on one fresh database produce exactly one admin row (test with two pools racing).
- SC-003: Repeated wrong logins produce `401`s, then a `429` whose `Retry-After` header matches the body wait; correct credentials during the block are refused; after clearing the throttle from the host, the next login succeeds.
- SC-004: The session id observed by the client before login differs from the one after login (fixation guard), and again after a password change.
- SC-005: Unknown-email and wrong-password login responses are byte-identical in body and status.
- SC-006: A password change with a 7-character password returns `400` with the minimum-length message; with a valid password the new one logs in and the rotation is observable per SC-004.
- SC-007: Workspace checks pass in one run: formatting, lint with warnings as errors, the full test suite, the module boundary check, and the frontend build.
- SC-008: The identity schema's reverse migration exits with an error and the database is byte-unchanged afterward.
- SC-009: A temporary password issued by the reset subcommand authenticates within 24 hours, is rejected after expiry, and each draw is uniform over its alphabet (statistical unit check).

## Edge Cases

- Concurrent setup across two connection pools on one fresh file → exactly one admin (FR-003).
- Instance crash between the schema seed and setup → a restart must not re-seed or block (idempotent seed insert).
- Login with correct credentials while blocked → still `429` (FR-007).
- Unknown-but-structurally-valid session id → unauthenticated, no error (FR-009).
- Session store outage → `500` with logged cause; never a silent logout (FR-009).
- Email variants differing only by case → treated as the same account (FR-014).
- Expired temporary password → authentication refused like any other invalid credential, with the shared `401`.
- Stored role corrupted on disk → typed error, no panic (FR-021).
- CLI run with the server's data directory missing → clear failure with cause chain, non-zero exit (FR-020).

## Assumptions

- Throttle identity is the submitted email normalized to lowercase; there is no IP-based dimension (single-user instances, privacy-first).
- "Admin" means role `admin`; exactly one admin exists per instance until multi-user milestones arrive.
- The block window is part of the recorded failure state, so a restart does not clear an active block.
- Temporary-password expiry is 24 hours; the must-change flag survives until the user completes a password change.
- Session timeout default of 7 days comes from configuration; operators can shorten but not bypass it.
- The `setup-status` endpoint may be called by unauthenticated clients (the onboarding UI needs it before any account exists); it exposes only a boolean.

## Out of Scope

- Self-registration (accounts are admin-driven only, per PRD).
- SMTP or any network-based password recovery (PRD optional, not configured).
- User profiles: display names, avatars — later milestone.
- Admin tooling beyond create: listing, editing, deleting users — deferred to the multi-user milestone.
- Static asset caching and HTTP shell concerns — owned by the web-api shell, not the identity capability.
- Multi-node deployments; the module targets a single host instance.

## Clarifications

### Session 2026-09-26

- Q: Which behavior should this spec formalize? → A: The delivered M1 identity module (PR #2), per user direction; this file is the single source of truth, superseding the earlier summary document.

## Deviations

None yet.
