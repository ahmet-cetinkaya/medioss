---
status: shipped
capability: profile-settings
plan: specs/0007-profile-settings/PLAN.md
supersedes:
---

# Profile Settings

## Intent

Every account can present itself: a display name and an avatar the owner controls. This closes the last PRD profile gap (`docs/PRD.md`: "display name and avatar") after theme preferences and email change were descoped on 2026-09-30.

The `/settings` page — seeded by spec 0004 with the self-delete danger zone — becomes the profile surface: display name editing, avatar upload/removal, and the existing password change. Visibility is deliberately narrow: the display name and avatar are shown to their owner everywhere in their own UI, and to the admin in the user list (management context). They are never shown member-to-member; the app has no social surface where they would appear.

Avatars are user-uploaded image files (jpg/png/webp, ≤ 2 MB), resized server-side and stored under `media/{user_id}/` — so user deletion (FR-027 of spec 0004) wipes them with everything else, for free.

## User Stories

### US1 — Member sets a display name (Priority: P1)

**Solo test:** As any user, open `/settings`; type a display name; save; the header greets me with that name.

- **Given** a session and an empty name field, **when** I save, **then** the API trims and stores the name (`200`), the UI updates immediately, and empty-after-trim clears the name back to unset.
- **Given** a name longer than 64 characters or consisting only of whitespace, **when** the form is submitted, **then** browser validation blocks it client-side and the API returns `400` with a field-level message.
- **Given** an anonymous session, **when** the profile endpoint is called, **then** the request gets `401`.

### US2 — Member uploads an avatar (Priority: P1)

**Solo test:** As any user, on `/settings`, pick a photo and upload; my avatar appears in the header within the same session.

- **Given** a jpg/png/webp file ≤ 2 MB, **when** it is uploaded, **then** the server stores a resized normalized copy under `media/{user_id}/` and returns `201`; the header and settings page show the new avatar without a page reload beyond the fetch.
- **Given** a file that is not a decodable image of an allowed type (magic bytes checked, not the client's claim), **when** it is uploaded, **then** the API returns `400` and no file is written.
- **Given** a file larger than 2 MB, **when** it is uploaded, **then** the request is rejected before full processing with `413` (or `400` per plan choice) and the UI shows the limit in the same language.
- **Given** SVG or any non-raster content, **when** it is uploaded, **then** the API rejects it (`400`); SVG is never accepted or served as an avatar (stored-XSS vector).

### US3 — Member removes their avatar (Priority: P2)

**Solo test:** As any user with an avatar, remove it; the header falls back to the initial-letter placeholder.

- **Given** an existing avatar, **when** the remove action is confirmed, **then** the file is deleted, the profile reports no avatar, and the UI shows the placeholder derived from the display name (or email when unset).
- **Given** no avatar exists, **when** remove is called, **then** the endpoint is idempotent (`204`).

### US4 — Admin sees display names and avatars in the user list (Priority: P2)

**Solo test:** As the admin, open the admin page; each row shows the account's display name (when set) and avatar (when set) next to the email.

- **Given** the admin session, **when** the user list loads, **then** each item carries `display_name` (nullable) and `has_avatar` (bool), rendered inline with the existing columns.
- **Given** a member session, **when** the list endpoint is called, **then** the response is `403` (unchanged contract from spec 0004).

## Requirements

### ADDED

- FR-030: WHEN a session owner submits a profile update (`PATCH /api/identity/me/profile`) with `display_name` THE SYSTEM SHALL trim it, persist it on the user row when 1–64 characters after trim, clear it when empty, and reject other shapes with `400`; the response returns the updated profile.
- FR-031: WHEN a session owner uploads an avatar (`PUT /api/identity/me/avatar`, multipart) THE SYSTEM SHALL validate size (≤ 2 MB) and decode the image (jpg/png/webp only, magic-byte sniffed), resize to a square ≤ 256 px (center crop), store the normalized copy under `media/{user_id}/`, delete any previous copy, and return `201`; any failure writes nothing.
- FR-032: WHEN a session owner deletes their avatar (`DELETE /api/identity/me/avatar`) THE SYSTEM SHALL remove the stored file and clear the flag, returning `204` idempotently.
- FR-033: WHEN a client fetches an avatar (`GET /api/identity/avatar/{user_id}`) THE SYSTEM SHALL serve it only to its owner or an admin session (`403` otherwise) with `Cache-Control: private, max-age` and the stored content type; a user without an avatar returns `404`.
- FR-034: WHEN `/api/identity/me` is served THE SYSTEM SHALL include `display_name` (nullable) and `has_avatar` in the response.
- FR-035: WHEN the admin list (`GET /api/identity/users`) is served THE SYSTEM SHALL include `display_name` and `has_avatar` per item, and the admin UI renders them per row.
- FR-036: WHEN a user is deleted (spec 0004 FR-027) THE SYSTEM SHALL remove the avatar with the rest of `media/{user_id}/` — no separate avatar cleanup path exists.

## Success Criteria

- SC-001: Route tests: `PATCH /api/identity/me/profile` → 200 owner (set, change, clear-by-empty) / 400 over-length / 400 whitespace-only / 401 anonymous; response carries the persisted profile.
- SC-002: Route tests: avatar `PUT` → 201 with a real small raster; `400` for oversized (>2 MB), corrupt bytes, wrong-type (e.g. text, SVG) payloads; `401` anonymous. Files land only under `media/{user_id}/`.
- SC-003: Route tests: avatar `GET` → 200 owner and admin sessions / `403` other member / `404` when unset; `DELETE` → 204 twice in a row (idempotent).
- SC-004: `GET /api/identity/me` and `GET /api/identity/users` expose `display_name` + `has_avatar`; proven by route tests (users list: admin only, member 403 per spec 0004).
- SC-005: Deletion integration: deleting a user removes their avatar file with the rest of `media/{user_id}/` (spec 0004's wipe test covers the avatar path too — extended assertion).
- SC-006: Re-upload replaces the previous file (exactly one avatar file per user remains under `media/{user_id}/avatar*`).
- SC-007: All gates green (fmt, clippy `-D warnings`, `cargo test --workspace`, UI build); live QA: upload from the browser → header shows avatar; remove → placeholder; display name change reflected in header and admin list.

## Edge Cases

- Display name is not unique; two users may share one. Nothing in the app treats it as an identifier.
- Unicode names (Turkish in particular) must survive trim/store/render round-trip; the length limit counts characters (UTF-8 scalars), not bytes.
- Concurrent avatar re-upload: each request stores, then unlinks the previous copy; worst case leaves one orphan file that user deletion wipes anyway — never two live files.
- Upload racing account deletion: either the file lands and is wiped by the delete sweep, or the row is gone and the orphan is wiped on the next sweep; no dangling reference is served (the avatar flag lives on the user row).
- A disabled account's avatar remains stored but unreachable (session 401 first) until re-enable or deletion.
- Trailing/leading whitespace in display name is stripped server-side; the UI also trims client-side so the two never disagree visibly.

## Assumptions

- The identity migration is amended in place (app unreleased; one-migration-per-module invariant): `users` gains `display_name TEXT NULL` and an avatar reference (`avatar_ext TEXT NULL` storing the normalized extension — the path is derived, `media/{user_id}/avatar.{ext}`). Dev databases re-initialize.
- Server-side resize/crop/encode uses the Rust `image` crate added to `web-api` only; no frontend image processing. The stored format is chosen by the plan (webp preferred, png fallback) and never echoes the client's.
- `media/{user_id}/` already exists as the per-user media root (meditation download cache, deletion wipe) — avatars reuse it without new lifecycle code.
- Endpoint paths/verbs are working contracts; the plan may adjust (e.g. `POST` vs `PUT` for the upload) as long as the SC contract holds.
- The `/settings` page (Turkish UI, spec 0004) hosts the profile section; the header shows avatar + display name for the logged-in user.
- The admin list rendering stays a simple table (no pagination); two extra columns do not change that.

## Out of Scope

- Built-in/preset avatar sets (user decision 2026-10-02: upload only).
- Theme preference, notification preferences, email change — descoped 2026-09-30; email stays the login identifier and is not editable.
- Avatars for anything other than self and the admin list (no member-to-member surface exists).
- Gravatar or any external avatar source; no outbound network fetches.
- Storage quota accounting (roadmap M6 item; separate spec when scheduled).

## Clarifications

### Session 2026-10-02 (spec creation)

- Avatar source: user file upload only (jpg/png/webp, 2 MB cap, server-side ~256 px normalize) — user decision over a built-in set or both.
- Visibility: owner's own UI (header, settings) + the admin user list; never member-to-member.
- Numbering: this is `0007`, not the provisional "0008" from `.agents/spec-gap-analysis.md`; spec 0007's originally-sketched scope (lifecycle) joined spec 0004 instead, freeing the number.

## Deviations

- FR-031/SC-002: the declared-size pre-check is not implemented; the running byte counter is the only size guard. axum 0.8 `Field` exposes no reliable declared length, and the counter rejects oversized payloads before decoding, preserving the contract (413, nothing written). Verified live: a 3 MB upload returns 413; multipart reader errors over the limit are also mapped to 413.
