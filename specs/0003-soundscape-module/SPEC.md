---
status: in_progress
capability: soundscape
plan: specs/0003-soundscape-module/PLAN.md
supersedes:
---

# M3 — Soundscape Module

## Intent

MediOSS users need a private, self-hosted way to combine ambient sounds into a personal soundscape for focus, relaxation, or sleep. This change introduces an admin-curated shared sound library, a mixer that can layer any number of library sounds at once with independent volume controls, reusable personal mixes, and a fade-out sleep timer. All audio and mix data remains local to the instance, and each user's saved mixes remain private to that user. The experience must remain stable during long playback and usable on both mobile and desktop without depending on an external media or analytics service.

## User Stories

### US1 — Admin curates the shared sound library (Priority: P1)

An admin can add, inspect, and remove ambient loops so signed-in users have a trusted sound library without managing files themselves.

**Independent test:** as an admin, upload a valid supported audio file with a name and category, see it in the library, stream it, then remove it; as a regular user, verify that all content-management actions are refused.

- **Given** an admin and a valid supported audio file within the size limit, **when** the admin submits a non-empty name and a category, **then** the sound is added to the shared library and becomes available to every signed-in user.
- **Given** a file whose declared extension or media type suggests audio but whose bytes are not supported audio, **when** the admin uploads it, **then** the upload is refused and no library item or retained media remains.
- **Given** a regular user, **when** they attempt to add, edit, or remove a library sound, **then** the action is refused without changing the library.
- **Given** an admin and an existing sound, **when** the admin removes it, **then** it is unavailable for future playback and is removed from saved mixes without deleting those mixes.

### US2 — Admin organizes sounds with categories (Priority: P1)

An admin can create, rename, and remove sound categories so the shared library reflects the instance's own organization rather than a fixed taxonomy.

**Independent test:** as an admin, create a category, rename it, assign a sound to it, verify deletion is refused while the category is in use, move the sound elsewhere, and then delete the empty category; as a regular user, verify every category mutation is refused.

- **Given** an admin, **when** they create a category with a unique non-empty name, **then** the category becomes available for sound uploads and library grouping.
- **Given** an existing category, **when** the admin renames it to a unique non-empty name, **then** every assigned sound remains in that category under its new name.
- **Given** a category assigned to one or more sounds, **when** the admin attempts to delete it, **then** deletion is refused and the category and assignments remain unchanged.
- **Given** an empty category, **when** the admin deletes it, **then** it is removed from future upload choices and library grouping.
- **Given** a regular user, **when** they attempt to create, rename, or delete a category, **then** the action is refused without changing categories.

### US3 — User creates and controls a live soundscape (Priority: P1)

A signed-in user can combine any number of library sounds, control each sound independently, and stop the whole soundscape immediately.

**Independent test:** select several different sounds, start them together, adjust each volume without interrupting the others, add another sound while playing and hear it join without restarting, and stop all playback immediately.

- **Given** a signed-in user and an available library, **when** they add distinct sounds to the mixer, **then** each selected sound is represented once with its own playback and volume control.
- **Given** a playing soundscape, **when** the user adds another sound, **then** the new sound starts playing immediately alongside the others without interrupting them.
- **Given** a playing soundscape, **when** the user changes one sound's volume, **then** that sound changes without restarting or changing the other sounds.
- **Given** a playing soundscape, **when** the user stops it, **then** every sound stops immediately and the active timer, if any, is cancelled.
- **Given** one or more selected sounds, **when** playback starts or loops, **then** each sound continues without an audible gap introduced by the application.

### US4 — User saves and restores personal mixes (Priority: P2)

A signed-in user can save a useful combination under a name and return to it later without exposing it to other users.

**Independent test:** save a multi-sound mix with distinct volumes, reload it in a new session, verify the same sounds and volumes are restored, and verify another user cannot list or load it.

- **Given** a mixer containing at least one sound, **when** the user saves it with a non-empty name, **then** a personal mix records the selected sounds and their individual volumes.
- **Given** a saved mix whose sounds still exist, **when** its owner loads it, **then** the mixer replaces its current selection with the saved sounds and volumes without starting playback automatically.
- **Given** a saved mix that references a sound later removed from the shared library, **when** its owner loads it, **then** the remaining sounds load and the user is told which sound is unavailable.
- **Given** one user, **when** they list, rename, load, or delete mixes, **then** only their own mixes are visible or mutable.

### US5 — User falls asleep with a fade-out timer (Priority: P2)

A signed-in user can schedule a playing soundscape to fade smoothly to silence and stop, without needing to return to the device.

**Independent test:** start a soundscape, choose an allowed timer duration, verify the remaining time is visible, verify volume decreases during the final fade period, and verify playback and the timer end at zero.

- **Given** a playing soundscape, **when** the user chooses a timer from 5 through 120 minutes, **then** the remaining time is shown and counts down while playback continues.
- **Given** an active timer, **when** it reaches its final minute, **then** the audible output fades progressively to silence before playback stops at zero.
- **Given** an active timer, **when** the user cancels it, **then** the soundscape keeps playing at the volumes selected before the fade began.
- **Given** an active timer, **when** the user replaces it with another allowed duration, **then** only the new timer remains active.

### US6 — User can leave the mixer running (Priority: P2)

A signed-in user can keep a soundscape playing during ordinary browser use without drift, unbounded resource growth, or accidental loss of controls.

**Independent test:** run a multi-sound soundscape continuously for at least one hour in a real supported browser, adjust volumes during playback, move the page between foreground and background where the platform permits, and verify playback and controls remain responsive.

- **Given** a three-sound soundscape, **when** it runs continuously for at least one hour, **then** playback remains audible and controls remain responsive without requiring a page reload.
- **Given** a playing soundscape, **when** the page becomes hidden or the device screen locks, **then** playback continues where the browser and operating system permit background audio.
- **Given** a playing soundscape, **when** the browser or operating system suspends audio despite the user's request, **then** returning to the mixer shows an accurate stopped or paused state rather than claiming playback continues.

## Functional Requirements

- **FR-001 (US1):** The system SHALL allow only an admin to add, edit, or remove shared sound-library items.
- **FR-002 (US1):** When an admin uploads a sound, the system SHALL require a non-empty name, one existing category, and a genuine supported audio payload no larger than 100 MB.
- **FR-003 (US1):** If a sound upload fails validation or storage, the system SHALL retain neither a library record nor media bytes from that attempt.
- **FR-004 (US1):** The system SHALL allow every signed-in user to list available sounds and obtain each sound's name, category, duration, and playback availability.
- **FR-005 (US1):** When a signed-in user requests an available sound's audio, the system SHALL support full and byte-range playback responses suitable for seeking and looping.
- **FR-006 (US1):** When an admin removes a sound, the system SHALL make its audio unavailable and remove its reference from all saved mixes while preserving the mixes and their remaining sounds.
- **FR-007 (US2):** The system SHALL allow only an admin to create, rename, or delete sound categories.
- **FR-008 (US2):** When an admin creates or renames a category, the system SHALL require a unique non-empty name and preserve all existing sound assignments on rename.
- **FR-009 (US2):** When an admin attempts to delete a category assigned to any sound, the system SHALL refuse the deletion without changing the category or its assignments.
- **FR-010 (US2):** When an admin deletes an empty category, the system SHALL remove it from future upload choices and library grouping.
- **FR-011 (US3):** While a user configures a live soundscape, the system SHALL permit between one and three distinct library sounds and SHALL reject a fourth without altering the existing selection.
- **FR-012 (US3):** The system SHALL provide an independent volume control from silent through full volume for every selected sound.
- **FR-013 (US3):** When a selected sound's volume changes, the system SHALL apply the change without restarting that sound or altering any other selected sound.
- **FR-014 (US3):** While a soundscape is playing, the system SHALL loop each selected sound continuously until the user stops it, the timer ends, the item becomes unavailable, or the platform suspends playback.
- **FR-015 (US3):** When the user stops playback, the system SHALL stop all selected sounds and cancel any active timer in one user action.
- **FR-016 (US3):** While a soundscape is playing, the system SHALL start a newly added sound immediately without interrupting the sounds already playing.
- **FR-016a (US3):** If one selected sound becomes unavailable during playback, the system SHALL stop that sound, keep the remaining sounds playing, and identify the unavailable sound to the user.
- **FR-017 (US4):** When a user saves a mix, the system SHALL require a non-empty name and at least one selected sound and SHALL record each selected sound with its current volume.
- **FR-018 (US4):** The system SHALL allow a user to list, load, rename, and delete only mixes they own.
- **FR-019 (US4):** When a user loads a saved mix, the system SHALL replace the current selection with the mix's available sounds and recorded volumes without starting playback automatically.
- **FR-020 (US4):** When a loaded mix references removed sounds, the system SHALL load the remaining sounds and identify every omitted sound.
- **FR-021 (US5):** While a soundscape is playing, the system SHALL allow the user to start or replace one fade-out timer with a duration from 5 through 120 minutes.
- **FR-022 (US5):** While a fade-out timer is active, the system SHALL show its remaining time and allow the user to cancel it.
- **FR-023 (US5):** When a fade-out timer enters its final minute, the system SHALL progressively reduce the combined audible output to silence and stop playback when the timer reaches zero.
- **FR-024 (US5):** If a user cancels a timer before it reaches zero, the system SHALL restore the pre-fade volume levels and continue playback.
- **FR-025 (US6):** While a soundscape is playing, the system SHALL keep the visible playback state synchronized with the browser's actual playing, paused, stopped, or failed state.
- **FR-026 (US6):** Where the browser and operating system permit background audio, the system SHALL continue an active soundscape when the page is hidden or the screen is locked.
- **FR-027 (US1, US4):** The system SHALL keep shared sound media and personal mix data within the self-hosted instance and SHALL not send playback, library, or mix data to an external service.
- **FR-028 (US1, US4):** When an unauthenticated client requests any soundscape category, library, audio, mixer-data, or saved-mix operation, the system SHALL refuse the request without disclosing private mix data.
- **FR-029 (US1):** The system SHALL support shipping a built-in sound set: at startup, if a built-in assets directory (override: `MEDIOSS_BUILTIN_SOUNDS_DIR`; defaults `builtin-sounds` beside the working directory and `/srv/medioss/builtin-sounds`) contains a `manifest.json`, the system SHALL import its categories and sounds idempotently — sounds are matched by a stable built-in identifier, re-runs make no duplicates, and imported media is stored lossless-preserving (FLAC for PCM sources, original bytes for compressed sources) and served exactly like admin uploads. Built-in sounds are ordinary library rows: an admin may still edit or remove them, and the API and UI SHALL mark them as built-in.

## Success Criteria

- **SC-001:** In an end-to-end test, an admin can upload, list, stream, and remove a valid sound, while equivalent write attempts by a regular user are refused.
- **SC-002:** In an end-to-end test, an admin can create and rename a category, cannot delete it while a sound uses it, and can delete it after the last sound is moved or removed; equivalent category mutations by a regular user are refused.
- **SC-003:** In automated authorization tests using two users, neither user can list, load, rename, or delete the other's saved mixes.
- **SC-004:** In a real supported browser, three sounds can play simultaneously for at least 60 minutes; all three remain audible, volume controls respond throughout, and the page requires no reload.
- **SC-005:** During a 60-minute three-sound run, the number of active sound players remains three and application memory does not grow continuously for the final 30 minutes.
- **SC-006:** In a mixer interaction test, changing one sound's volume affects only that sound and does not restart any selected sound.
- **SC-007:** In an end-to-end timer test using an accelerated test clock, the final fade reaches silence and all playback stops when the timer reaches zero; cancellation restores the pre-fade volumes.
- **SC-008:** Loading a saved mix restores all available selected sounds and their volumes exactly, starts no playback, and reports any removed sounds.
- **SC-009:** A network inspection of category, upload, library, playback, mixer, and saved-mix flows shows no request to a service outside the MediOSS instance.
- **SC-010:** The mixer can be fully operated at 320 CSS pixels wide and at desktop width without horizontal scrolling, clipped controls, or an inaccessible emergency-stop action.
- **SC-011:** All interactive mixer and category-management controls expose an accessible name, are keyboard operable, have a visible focus indicator, and meet WCAG 2.1 AA contrast requirements.

## Edge Cases

- The shared library is empty: the mixer explains that an admin must add sounds and offers no broken playback controls.
- The user opens a saved mix after one or more referenced sounds were removed: available sounds load; removed sounds are named and omitted.
- The same sound is selected twice: the existing selection is focused or retained rather than duplicated.
- A fourth sound is requested: the existing three remain unchanged.
- A sound fails midway through playback: other sounds continue and the failed sound is identified.
- The user changes volume during the final fade: the fade continues toward silence while the user's chosen relative balance is retained.
- The user cancels or replaces a timer during the fade: the new state takes effect once and stale timers cannot later stop playback.
- The page is refreshed or closed while playback is active: playback ends; the selected but unsaved live mix is not promised to survive.
- The browser blocks autoplay: the mixer waits for an explicit user gesture and clearly shows that playback has not begun.
- Network connectivity to the self-hosted instance is interrupted after audio has begun: already buffered playback may continue, but the interface reports failures when further audio data cannot be obtained.
- A sound has no detectable duration or cannot loop reliably: it is rejected during curation rather than entering the shared library.
- Two devices load the same saved mix: each device controls its own live playback; saving or renaming changes the stored mix for the next load without remotely controlling the other device.
- An admin attempts to create or rename a category to an existing name: the request is refused and existing categories remain unchanged.
- An admin attempts to delete a category that still contains sounds: deletion is refused and no sound is left without a category.
- Two admins concurrently create the same category name: exactly one succeeds and one receives a duplicate response.

## Assumptions

- The built-in sound set ships from the [Moodist](https://github.com/remvze/moodist) project; per-file licenses and upstream sources are recorded in `assets/sounds/soundscape-builtin/LICENSE` (Pixabay Content License / CC0; bundled as application content, not redistributable standalone). Built-in set storage: WAV sources are stored as FLAC (bit-exact lossless) and MP3 sources are re-encoded once to Opus VBR at the transparency tier (160 kbps stereo / 112 kbps mono). User uploads remain lossless-preserving: PCM sources (WAV) are stored as FLAC (bit-exact), already-compressed sources (MP3, Ogg Opus) are stored byte-identical, and no upload is ever re-encoded lossily.
- The roadmap's M3 follows M2 and is the next new capability; this specification therefore follows the repository's established milestone convention as `specs/0003-soundscape-module/SPEC.md`.
- The library follows the current product direction established by M2: admins curate shared media, while all signed-in users consume it. “Custom upload” in the PRD means instance-local custom sounds curated by the admin in this milestone, not per-user uploads.
- Supported upload formats and the per-file maximum match the current local-audio contract: genuine MP3, WAV, or OGG audio, at most 100 MB.
- Every library sound has exactly one required admin-managed category; category names are instance-wide unique and may be created, renamed, and deleted when empty.
- One live soundscape contains at most three distinct sounds, using the stricter roadmap limit rather than the PRD's broader 3–5 range.
- Saved mixes are private per-user preferences. The live mixer state and timer are local to the current browser session and are not synchronized across devices.
- A saved mix stores sound references and per-sound volumes only; timer state and current playback position are not part of a mix.
- The fade-out period is the final minute of an allowed 5–120 minute timer. Timer durations may be offered as preset choices as long as every offered value remains within that range.
- Background and lock-screen playback is best-effort because browsers and operating systems may suspend audio; the system must report actual state rather than promise platform behavior it cannot control.
- Continuous playback for this milestone is measured at the roadmap's one-hour exit criterion. The PRD's six-hour target remains a later hardening goal.

## Out of Scope

- Per-user sound uploads or private sound libraries.
- More than three simultaneous sounds.
- Sharing mixes with other users or publishing community presets.
- Category ordering, nesting, icons, tags, cover art, descriptions, or search/filter beyond category grouping.
- Recording, synthesizing, generating, or editing audio inside MediOSS.
- Crossfading between mixes, equalizers, spatial audio, normalization, or per-sound playback speed.
- Persisting or synchronizing live playback position, active timers, or play/pause state across sessions or devices.
- External streaming catalogs, cloud storage, telemetry, advertising, or third-party analytics.
- Guaranteed background playback on a browser or operating system that suspends web audio.
- Offline installation or caching of sound audio for playback without access to the instance.
- Six-hour endurance certification; M3 certifies the roadmap's one-hour target.

## Clarifications

No clarifications recorded yet.
