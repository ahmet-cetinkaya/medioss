---
status: in_progress
capability: meditation
plan: specs/0002-meditation-module/PLAN.md
supersedes:
---

# M2 — Meditation Module

## Intent

An instance maintains a shared meditation library that only the admin curates: they upload personal audio (genuinely audio, size-capped, converted to Opus for efficient storage), organize it into programs with cover images under categories, and every signed-in user can browse, stream with seeking, and download it. Each user's practice data stays their own: playback position, completed state, meditation sessions, practice notes, and program progress are recorded per user and are invisible to others. This delivers the meditation pillar of the MVP per the PRD and roadmap M2, reshaped by user direction toward an admin-curated shared library for the first stage, on top of the identity capability (specs/0001-identity-module): every meditation action is performed by an authenticated Actor. Stored media bytes live only under the operator-configured data directory, so backing up that directory captures everything.

## User Stories

### US1 — Admin uploads meditation audio (Priority: P1)

The admin adds an audio file (MP3, WAV, or OGG) with a name they have entered — prefilled from the file name in Title Case, not taken verbatim; regular users cannot add content.

**Independent test:** as admin a valid small MP3 uploads and appears in the shared library; a request declaring more than 100MB is refused without its body being read; random bytes named `.mp3` are refused as non-audio; a regular user's upload is refused with 403.

- Given the admin, when they upload at most 100MB of genuine MP3, WAV, or OGG audio with a non-empty name, then the meditation is stored and its metadata (identifier, name, content type, size, creation time) is returned.
- Given the upload form with a chosen file, when the name field is presented, then it is prefilled from the file name in Title Case and remains editable before submission.
- Given a request whose declared size exceeds 100MB, when the request arrives, then it is rejected before the body is read.
- Given an upload whose actual byte stream exceeds 100MB during transfer (undeclared or understated length), when the excess is detected, then the upload is aborted and nothing is stored.
- Given an upload whose bytes are not genuinely one of the accepted formats, when the file name or declared content type claims otherwise, then it is rejected.
- Given a regular (non-admin) user, when any content-management route is called, then the response is 403.
- Given a requester without a session, when any meditation route is called, then the response is 401.

### US2 — Shared library list (Priority: P1)

Every signed-in user sees the same shared library of meditations; there are no per-user libraries in this stage.

**Independent test:** the admin uploads one file; a regular user's library list contains exactly that item; the admin's list is identical.

- Given any authenticated user, when they list the library, then all meditations in the shared library are returned, and an empty list when none exist yet.
- Given any authenticated user, when they request one meditation's metadata by identifier, then that metadata is returned.

### US3 — Authenticated streaming with seeking (Priority: P1)

Any signed-in user streams library audio, including byte-range requests for seeking; unauthenticated requesters get nothing.

**Independent test (roadmap exit criterion):** the admin uploads an MP3; a regular user requests it with a Range header and receives 206 with the requested bytes; an unauthenticated requester receives 401; a nonexistent identifier receives 404.

- Given any authenticated user requesting audio without a Range header, when the file exists, then the complete byte stream is returned with 200 and the true content type.
- Given any authenticated user sending a well-formed satisfiable Range (including open-ended and suffix forms), when the file exists, then exactly the requested bytes are returned with 206 and a Content-Range header.
- Given any authenticated user sending a well-formed Range whose start lies beyond the end of the file, then 416 is returned.
- Given a malformed Range header, when audio is requested, then it is ignored and the full stream is returned with 200.
- Given any requester naming an identifier that does not exist, when a meditation resource is requested, then 404 is returned.

### US4 — Admin deletes a meditation (Priority: P1)

The admin removes a meditation entirely: metadata record, stored bytes, its program memberships, and every user's notes on it.

**Independent test:** upload, delete, then the library omits it, streaming returns 404, no bytes remain under the data directory, and notes on it are gone; a regular user's delete attempt returns 403.

- Given the admin deletes a meditation, when the deletion completes, then the metadata record, every stored byte for it, its program memberships, and all notes on it (all users') are removed.
- Given a regular user, when delete is attempted, then 403 is returned.
- Given any requester naming an identifier that does not exist, when delete is attempted, then 404 is returned.

### US5 — Playback with per-user resume (Priority: P1)

In the web UI any user plays, pauses, seeks, changes speed from 0.5x to 2.0x, and on returning to a partially played meditation continues where they left off — each user at their own position.

**Independent test:** user A plays 30 seconds of a 10-minute file in a real browser, reloads, reopens — playback resumes near the 30-second mark; user B opening the same meditation starts at 0.

- Given a user plays a meditation, when the client reports the playback position, then the position is persisted for that user and that meditation and is retrievable.
- Given a user with a stored position below the meditation's end, when they reopen it, then playback starts at the stored position.
- Given a user whose completed state for the meditation is true, when they reopen it, then playback starts from the beginning.
- Given the playback UI, then play, pause, seek, and speed selection between 0.5x and 2.0x are operable in a real browser.

### US6 — Per-user completion recording and state (Priority: P1)

When a user plays a meditation to the end, the instance records a meditation session with start and finish timestamps and marks the meditation as completed for that user. Each user sees and changes only their own completed state.

**Independent test:** user A finishes playback — the acknowledgement carries start and finish timestamps and the meditation shows as completed for A; user B sees it as not completed; A toggles it back without creating or removing any session.

- Given a user's client reports completion of a playback pass, when the report is accepted, then one meditation session with start and finish timestamps is recorded and acknowledged.
- Given a duplicate completion report for a playback pass whose completion was already recorded, when it arrives, then no second meditation session is created.
- Given playback completes for a user, when the completion is recorded, then that user's completed state for the meditation is set to true.
- Given a user, when they toggle their own completed state for a meditation in either direction, then the state changes accordingly without creating or removing any meditation session.
- Given user A's completed state, when user B views the same meditation, then B sees only their own state, never A's.

### US7 — Programs: curated shared lists (Priority: P1)

The admin collects library meditations into programs: each program has a title, an optional description, and an optional cover image; every user sees the same programs, and each user sees their own completion progress on them.

**Independent test:** a program with three meditations, one of them completed by user A, shows 33% for A; user B sees 0% on the same program; its cover is visible to every authenticated user.

- Given the admin, when they create a program with a non-empty title (description and cover image optional), then the program is stored and its metadata returned.
- Given a program cover image is supplied, when its bytes are a genuine accepted image format within the cover size cap, then it is stored like audio and served to authenticated users.
- Given the admin, when they add or remove a meditation to/from a program, then the program's membership list reflects it; membership order is the order meditations were added.
- Given the admin, when they delete a program, then the program and its memberships are removed; meditations, bytes, per-user data, and notes remain intact.
- Given a program with at least one member meditation, when a user views it, then the progress shown equals that user's share of member meditations whose completed state (theirs) is true.
- Given an empty program, when it is displayed, then no progress fraction is shown rather than a misleading 0%.
- Given a regular user, when any program-management route is called, then 403 is returned.

### US8 — Program categories (Priority: P2)

The admin creates categories and assigns one to each program; the library view groups program cards under category headings.

**Independent test:** programs with categories "Morning" and "Evening" appear under those headings; a program without category appears under a trailing uncategorized heading.

- Given the admin, when they create a category with a non-empty name unique among existing categories, then it is stored.
- Given the admin, when they assign a category to a program, then the program appears under exactly that category heading.
- Given the admin, when they delete a category, then its programs become uncategorized and are shown under the uncategorized heading.
- Given the library view, when programs are listed, then program cards are grouped under category headings, with uncategorized programs last.
- Given a regular user, when any category-management route is called, then 403 is returned.

### US9 — Per-user practice notes (Priority: P2)

After practicing, a user writes notes linked to the meditation for their own reflection; notes are private to their author even though the library is shared.

**Independent test:** after a playback, user A writes a note on that meditation; it appears in A's note list; user B's list for the same meditation is empty and every route touching A's note returns 404 for B.

- Given an authenticated user, when they create a note on a meditation, then the note is stored with its creation time, linked to that meditation and that author.
- Given a user, when they list their notes on a meditation, then all their own notes for it are returned newest first, and no other user's notes.
- Given the author, when they edit or delete their note, then the change is applied to exactly that note.
- Given any user other than the author, when a route addressing another user's note is called, then 404 is returned, indistinguishable from a nonexistent identifier.

### US10 — Efficient storage via Opus conversion (Priority: P1)

Accepted uploads are stored once, as Opus, so disk usage stays efficient; the original file is not retained.

**Independent test:** upload a large WAV; the stored file is Ogg Opus, substantially smaller, and streams and plays back end-to-end.

- Given an accepted upload in any accepted format, when storage completes, then the stored file is Opus in an Ogg container and the original bytes are not retained.
- Given a WAV source, when converted, then the Opus encoding is transparent at the configured quality setting and the stored size is substantially smaller than the source.
- Given an upload that is already Opus in an Ogg container, when stored, then it is kept as-is without re-encoding.
- Given a stream that passes initial signature validation but cannot be decoded fully (corrupt mid-stream), when conversion fails, then the upload is rejected and nothing is stored.

### US11 — Download meditation (Priority: P2)

Any signed-in user can download a library meditation to their device.

**Independent test:** a regular user's download returns the stored audio as an attachment; an unauthenticated requester receives 401; a nonexistent identifier receives 404.

- Given an authenticated user, when they request a download of a meditation, then the stored audio is returned as a file attachment with a filename derived from the meditation's name.
- Given any requester naming an identifier that does not exist, when a download is attempted, then 404 is returned.

## Requirements

### ADDED

- FR-001: WHEN the admin uploads at most 100MB of genuine MP3, WAV, or OGG audio with a non-empty name THE SYSTEM SHALL store the meditation and return its metadata (identifier, name, content type, size, creation time).
- FR-002: WHEN an upload request declares a size above 100MB THE SYSTEM SHALL reject it with 413 before reading the request body.
- FR-003: WHEN an upload's actual byte stream exceeds 100MB during transfer THE SYSTEM SHALL abort and reject the upload without storing any record or byte.
- FR-004: WHEN uploaded bytes are not genuinely one of the accepted audio formats THE SYSTEM SHALL reject the upload with 415 regardless of the file name or declared content type.
- FR-005: WHEN a regular user calls a content-management route (upload, delete meditation, program or category management) THE SYSTEM SHALL respond with 403.
- FR-006: WHEN a request reaches any meditation route without an authenticated session THE SYSTEM SHALL respond with 401.
- FR-007: WHEN an authenticated user lists the library THE SYSTEM SHALL return the shared library's meditations, identical for every user, and an empty list when none exist.
- FR-008: WHEN an authenticated user requests one meditation's metadata by identifier THE SYSTEM SHALL return it.
- FR-009: WHEN an authenticated user requests a meditation's audio without a Range header THE SYSTEM SHALL stream the entire file with 200 and its true content type.
- FR-010: WHEN an authenticated user requests audio with a well-formed satisfiable Range (including open-ended and suffix forms) THE SYSTEM SHALL return exactly the requested bytes with 206 and a Content-Range header.
- FR-011: WHEN an authenticated user requests audio with a well-formed Range whose start lies beyond the end of the file THE SYSTEM SHALL return 416.
- FR-012: WHEN an authenticated user requests audio with a malformed Range header THE SYSTEM SHALL ignore it and return the full stream with 200.
- FR-013: WHEN a meditation resource is requested for an identifier that does not exist THE SYSTEM SHALL return 404.
- FR-014: WHEN the admin deletes a meditation THE SYSTEM SHALL remove the metadata record, every stored byte for it, its program memberships, and all users' notes on it.
- FR-015: WHEN a user reports a playback position for a meditation THE SYSTEM SHALL persist that position per user per meditation.
- FR-016: WHEN a user queries their playback state for a meditation THE SYSTEM SHALL return their stored position, or zero when they never played it.
- FR-017: WHILE the web UI plays a meditation THE SYSTEM SHALL provide play, pause, seek, and playback speed between 0.5x and 2.0x.
- FR-018: WHEN a user reopens a partially played meditation THE SYSTEM SHALL start playback at that user's stored position.
- FR-019: WHEN a user's completed state for a meditation is true and they reopen it THE SYSTEM SHALL start playback from the beginning.
- FR-020: WHEN a user's client reports completion of a playback pass THE SYSTEM SHALL record exactly one meditation session carrying start and finish timestamps, acknowledge it, and set that user's completed state to true.
- FR-021: WHEN a completion report arrives for a playback pass whose completion was already recorded THE SYSTEM SHALL record no additional meditation session.
- FR-022: WHEN a user toggles their completed state for a meditation THE SYSTEM SHALL change that state as requested without creating or removing any meditation session.
- FR-023: WHEN any per-user practice data (position, completed state, meditation sessions, notes, progress) is read or written THE SYSTEM SHALL scope it to the acting user only.
- FR-024: WHEN the upload form presents a chosen file THE SYSTEM SHALL prefill the name field from the file name in Title Case and keep it editable.
- FR-025: WHEN the admin creates a program with a non-empty title THE SYSTEM SHALL store it with its optional description and optional cover image and return its metadata.
- FR-026: WHEN a program cover image is supplied THE SYSTEM SHALL accept it only if its bytes are a genuine accepted image format within the cover size cap, and store it and serve it to authenticated users like audio.
- FR-027: WHEN the admin adds or removes a meditation to/from a program THE SYSTEM SHALL update the membership, ordered by when meditations were added.
- FR-028: WHEN the admin deletes a program THE SYSTEM SHALL remove the program and its memberships and retain all meditations, bytes, per-user data, and notes.
- FR-029: WHEN a program's progress is displayed to a user THE SYSTEM SHALL show that user's share of member meditations whose completed state is true, and show no fraction for an empty program.
- FR-030: WHEN the admin creates a category with a non-empty name unique among existing categories THE SYSTEM SHALL store it.
- FR-031: WHEN the admin assigns a category to a program THE SYSTEM SHALL group that program under exactly that category.
- FR-032: WHEN the admin deletes a category THE SYSTEM SHALL leave its programs intact and uncategorized.
- FR-033: WHEN the library view lists programs THE SYSTEM SHALL group program cards under category headings with uncategorized programs last.
- FR-034: WHEN an authenticated user creates a note on a meditation THE SYSTEM SHALL store it linked to that meditation and that author with its creation time.
- FR-035: WHEN a user lists notes on a meditation THE SYSTEM SHALL return only that user's own notes for it, newest first.
- FR-036: WHEN the author edits or deletes their note THE SYSTEM SHALL apply the change to exactly that note.
- FR-037: WHEN a route addressing another user's note is called by anyone but its author THE SYSTEM SHALL return 404 indistinguishable from a nonexistent identifier.
- FR-038: WHEN an accepted audio upload is stored THE SYSTEM SHALL convert it to Opus in an Ogg container at a transparent quality setting and persist only the converted form.
- FR-039: WHEN an accepted upload is already Opus in an Ogg container THE SYSTEM SHALL store it as-is without re-encoding.
- FR-040: WHEN an authenticated user requests a download of a meditation THE SYSTEM SHALL return the stored audio as a file attachment named after the meditation.

### PRESERVED

- PB-1: The identity capability keeps behaving exactly as specified in specs/0001-identity-module: setup, login with throttling, session lifecycle, password change, and admin user creation keep their E2E suite green after meditation routes are mounted alongside them.
- PB-2: The shared media storage behavior shipped in M0 keeps its guarantees: blobs are written, opened, and deleted under generated opaque keys inside the data directory, and no user-controlled string ever reaches a stored path.
- PB-3: Unauthenticated requests keep receiving 401 through the same authentication path identity uses; meditation adds no unauthenticated data path.

## Success Criteria

- SC-001: E2E against the real binary in CI: the admin uploads a small MP3, then a regular user requests the audio with `Range: bytes=0-99` and receives `206` with exactly 100 bytes and a valid `Content-Range`; an unauthenticated requester receives `401`; a nonexistent identifier receives `404`.
- SC-002: An upload declaring more than 100MB is rejected with 413 before its body is consumed — the response is produced while the declared body has not been read.
- SC-003: A file of random bytes named `fake.mp3` is rejected with 415, and afterwards the library count and stored byte count are unchanged (nothing persisted).
- SC-004: A regular user's upload, delete, and program/category management calls all return 403; the admin's equivalents succeed.
- SC-005: With users A and B: A completes one of three meditations in a program and writes a note; A's program progress shows one third and B's shows zero on the same program; B's note list for that meditation is empty and every route touching A's note returns 404 for B; both users' library lists are identical.
- SC-006: After the admin deletes a meditation, the library omits it, its audio request returns 404, all users' notes on it are gone, and zero bytes remain for it under the data directory's media area.
- SC-007: Browser QA: play a meditation of at least 5 minutes, seek, set 1.5x speed, play about 30 seconds, reload the page, reopen — playback resumes within 2 seconds of the stored position; after playing to the end and reopening, playback starts at 0.
- SC-008: Completing playback returns an acknowledgement whose finish timestamp is greater than its start timestamp, exactly one meditation session exists for that playback pass after one retry of the completion report, and the completing user's completed state is true.
- SC-009: Workspace checks pass in one run: formatting, lint with warnings as errors, the full test suite, the module boundary check, and the frontend build.
- SC-010: A manual QA checklist covering play, pause, seek, both speed bounds (0.5x and 2.0x), and resume passes with every item checked in a real browser — the roadmap M2 exit criterion.
- SC-011: A program with three member meditations reports progress of one third for a user who completed exactly one, and no fraction when empty; completing a meditation that belongs to two programs raises both programs' progress for that user.
- SC-012: Browser QA: the library view shows program cards grouped under category headings with uncategorized programs last, each card showing title, cover or placeholder, and the viewing user's progress.
- SC-013: Uploading a file named `morning_meditation.mp3` prefills the name field as `Morning Meditation`, editable before submission; the stored meditation uses the submitted name.
- SC-014: Notes created after playback are listed newest first under their meditation, editable and deletable by the author only; another user receives 404 for every route addressing them.
- SC-015: Toggling a completed meditation to not-completed and back changes the state both times and leaves the count of meditation sessions unchanged.
- SC-016: A large WAV uploads successfully; the stored file is Ogg Opus (`audio/ogg`) and substantially smaller than the source, and it streams and plays in a real browser end-to-end.
- SC-017: A regular user's download returns the stored Opus file with an attachment disposition and a filename derived from the meditation's name; an unauthenticated requester receives 401.

## Edge Cases

- Upload aborted by the client mid-body → no partial record, no orphan bytes (with FR-003's abort path).
- Declared Content-Length smaller than the actual body → mid-stream cap enforcement rejects (FR-003).
- Upload of exactly 100MB → accepted; the cap is inclusive (FR-001).
- Zero-byte upload → rejected as non-audio via content validation (FR-004).
- Two byte-identical uploads → stored independently under distinct opaque keys (PB-2).
- Two meditations with the same name → both kept; names are not unique.
- Concurrent range requests on one file → all succeed; the file is never modified by reads.
- Delete while a stream is in flight → deletion does not block on active streams; after it completes, new requests return 404.
- A position report arriving after completion → the report belongs to a new playback pass; the stored position becomes that report's value, superseding the completion reset.
- Completion reported with no prior position reports → accepted and recorded: position reporting is optional and the report itself is authenticated like every meditation call.
- A meditation in two programs completed once by a user → both programs' progress count it for that user; the completed state belongs to the user-meditation pair, not the membership.
- Cover image that is oversized or not a genuine accepted image → program creation proceeds only after the cover is valid or omitted.
- Deleting a meditation that sits in programs and carries notes → memberships and all users' notes disappear with it; programs survive with remaining members.
- Deleting a category still holding programs → programs remain, now uncategorized.
- Name prefill for a file name with no letters (e.g. `123.mp3`) → the prefill is the digits as-is; the user can always edit.
- Duplicate category name → rejected; category names are unique instance-wide because categories are admin-owned shared structure.
- A regular user's position/progress/notes for a meditation deleted by the admin → removed with the meditation (nothing dangles); their data for other meditations is untouched.
- A stream that passes signature validation but fails full decode → conversion fails → rejected with nothing stored (with FR-038's pipeline).
- An upload that is already Opus in Ogg → stored byte-for-byte, no re-encode (FR-039).

## Assumptions

- Content management (upload, meditation delete, program and category CRUD) is restricted to the admin role; regular users read the shared library and manage only their own practice data. Per-user uploads arrive in a later milestone.
- Accepted upload types are MP3, WAV, and OGG, validated from the actual bytes (format/container signature), per the PRD; the 100MB per-file cap comes from the PRD and applies to the uploaded original (the converted Opus is naturally smaller).
- Opus is a lossy codec: "without quality loss" is implemented as transparent-quality encoding (inaudible in listening). WAV sources get a transparent encode; MP3/Vorbis sources necessarily take one additional lossy generation, kept inaudible by a high-quality setting. The exact encoder settings are a plan concern.
- The stored form is Ogg Opus, served as `audio/ogg`; target browsers are modern Chromium/Firefox and Safari 17+ (Ogg Opus playback support).
- Metadata reports the stored (converted) content type and size; the original's format and size are not tracked after conversion.
- Download serves the stored Opus file to any authenticated user; the filename is derived from the meditation's name (transliterated/sanitized) with an `.opus` extension.
- The meditation's display name is free text, non-empty, not unique; the prefill transform strips the extension, replaces separator characters with spaces, and capitalizes each word (Title Case).
- Description and note text are free text, possibly empty; a program's description is optional and may be left blank.
- Accepted cover image types are PNG, JPEG, and WebP, validated from bytes, with a 5MB cap; covers are served to authenticated users like audio.
- A meditation may belong to any number of programs; membership order is the order meditations were added, and manual reordering is out of scope for M2.
- A program carries at most one category; programs without a category appear under a single uncategorized heading, listed last.
- Category names are unique instance-wide (categories are shared structure, not per-user).
- The completed state is a single boolean per user per meditation; completing playback sets it to true for that user, who may toggle it either way at any time; toggling never touches meditation sessions.
- "Meditation session" means one completed playback pass (start and finish timestamps); it is distinct from the identity login session and carries nothing else in M2 — history views and statistics are M7.
- Notes are per user per meditation, any number, newest first; the note body is free text.
- Server-side duration and bitrate extraction is out of M2; the player derives duration from the audio itself during playback.
- Playback position is reported by the client (periodically and on pause/close), measured in media time so speed changes do not distort it.
- Completion means the client reports that playback reached the end of the media; the report is authenticated like every other meditation call.
- Completion resets the stored position to zero for that user, which is why completed meditations restart from the beginning.
- Meditation resources are mounted under the same composition root as identity routes, following the identity naming precedent (`/identity/...` → `/meditation/...`); exact paths are a plan concern.
- Every meditation route requires an authenticated Actor; there is no public meditation surface.
- Streak counting, sleep timers, auto-play, offline caching, and history views belong to later milestones (M3/M7/M8).

## Out of Scope

- Per-user uploads and per-user libraries — a later milestone; this stage has one admin-curated shared library.
- Reordering program members manually (drag-and-drop) — later milestone.
- More than one category per program, or categories on meditations themselves (categories apply to programs only).
- Editing meditation names after upload, editing audio files — not in roadmap M2.
- Search and filtering, streak counting, total-time statistics, calendar view, playback history UI (M7).
- Sleep timer and fade-out (M3/M7).
- Auto-play next in program (M7).
- Offline/PWA access to media (M8).
- Retaining original files alongside converted copies (conversion to Opus is in scope); waveform or album-art generation.
- Sharing programs with other users or exporting programs.
- The soundscape module and its uploads (M3), though it will reuse the shared media storage.
- Seed meditation content and sourcing documentation (M5).
- Account-deletion media wipe (M6); M2 deletes individual meditations only.
- Server-side duration/bitrate metadata extraction.

## Clarifications

### Session 2026-09-26

- No open questions survived repository discovery: the roadmap M2 section, the PRD meditation pillar, and the architecture media-security contract settled scope, limits, and vocabulary. All defaults taken are recorded under Assumptions; no `[NEEDS CLARIFICATION]` markers remain.

### Session 2026-09-26 (user direction, spec still draft)

- Q: Should meditations be collectable into programs? → A (user): Yes — programs (meditation lists) with title, optional description, optional cover image; categories user-created and assignable to programs; library view groups program cards under category headings; per-meditation practice notes; a user-visible and user-toggleable completed field set by playback completion; program progress as the share of completed meditations; uploaded files get a user-entered name prefilled from the file name in Title Case.

### Session 2026-09-26 (user direction 2, spec still draft)

- Q: Storage format and downloads? → A (user): Accepted uploads are converted to Opus for efficient storage ("without quality loss" recorded honestly as transparent-quality encoding, since Opus is lossy — see Assumptions) and users can download a meditation to their device.

### Session 2026-09-26 (user direction 3, spec still draft)

- Q: Who owns library content? → A (user): First stage is admin-curated: only the admin uploads and manages meditations, programs, and categories; every user sees the shared meditations and programs; progress, notes, and other practice data stay per-user. This reshaped US1–US9, FR-001–FR-037 into the shared-library model (regular-user management calls 403; per-user scoping for position, completed state, sessions, notes, and progress via FR-023); category uniqueness became instance-wide because categories are shared structure.
