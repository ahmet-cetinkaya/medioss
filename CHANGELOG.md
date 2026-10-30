# Changelog

All notable changes to MediOSS are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning is semantic (`MAJOR.MINOR.PATCH`), and only stable tags publish a Docker image (`latest` tracks the last stable release).

## [Unreleased]

### Fixed

- **The change-password dialog no longer closes on Escape or an outside click while the request runs**, matching the other form dialogs.

- **Mobile bottom bar had extra space under it while the browser's top bar was shown** (Firefox on Android; the cause is suspected, not yet confirmed on a device). The bar now ignores `env(safe-area-inset-bottom)`, which that browser fills with its toolbar height although the page does not opt in to edge-to-edge drawing.

### Changed

- **Mobile player controls fit on two rows.** Row 1: back 10 s, play/pause, forward 10 s. Row 2: restart, speed, and an icon-only Soundscape button (its name still says when a soundscape is playing). On screens up to 700 px tall the cover is wider and shorter, so every control fits above the bottom bar (checked at 360x640 and 390x700).

- **Admin: Users is the last tab**, and **Create user** is a dialog opened from a button at the top of the user list. The one-time temporary password still shows above the list.
- **Settings: no always-open forms.** Changing the password is a dialog opened from a **Change password** button in the profile section. Importing data is a dialog opened from an **Import** button, and deleting the account is a dialog (email confirmation inside) opened from a **Delete account** button.

- **Program meditation cards open the meditation.** The whole row is the link, like the Home cards, not just the play button.
- **Today's suggestion shows its program** instead of the duration (a meditation without a program still shows the duration). `GET /api/meditation/daily-suggestion` now returns `program` (id, title, has_cover) when the meditation has one.
- **Mood history cards show the score's face**, the same one used when picking the score.

- **Filters are one row.** Library, Mood and Soundscape use a dropdown instead of wrapping chips; on Library the dropdown sits next to the search box in the header. History puts its date range and "Notes only" behind one filter button.
- **Program page, desktop:** the session list is on the left; cover, description and progress are in a glass section on the right. On mobile the order is unchanged and the start button sits under the description.

## [1.1.4] — 2026-10-30

### Added

- **Searchable meditation picker.** "Add existing meditation" now opens a search box (by meditation or program name, ignoring case and accents) over a card list that shows each meditation's duration and every program it belongs to. Clicking a card adds it.

- **Add to another program.** The meditation menu in a program has a new "Add to another program" entry that opens a searchable program list. The "Add to a program" dialog for meditations without a program uses the same list.

### Changed

- `GET /api/meditation/library` now returns `programs` (every program a meditation belongs to) on each meditation.

## [1.1.3] — 2026-10-30

### Added

- **Rename a meditation** from the admin UI (`PUT /api/meditation/meditations/{id}/name`, admin only). The menu entry sits next to "Edit description", both for meditations in a program and for those without one.

- **Remove a program's cover** (`DELETE /api/meditation/programs/{id}/cover`, admin only, idempotent): the program edit dialog shows "Remove the cover" while a cover is set.

### Fixed

- **Sound upload dialog** no longer scrolls sideways: the hidden icon file input was dialog-wide because the styled input's width overrode `sr-only`.
- **Deleting a program** leaves its detail view only after the delete succeeded; a failed delete no longer navigates away.
- **A deep link to a program that no longer exists** falls back to the programs list instead of loading nothing.
- **Meditations without a program** can now be deleted and edited from the admin menu; 1.1.2 only offered play and "Add to a program" there.

## [1.1.2] — 2026-10-29

### Added

- **Delete programs and meditations from the admin UI** (`#106`): the overflow menu and detail view of a program get "Delete program", and each meditation in a program gets "Delete meditation". Both ask for confirmation first; deleting a program keeps its meditations in the library. The endpoints already existed.

### Fixed

- **Deep link to a program** (`/admin/meditations/<id>/`) no longer sticks on "Loading…": the meditations list loads on first open.

### Security

- **Dependency advisories** (`#102`): `rustls-webpki` 0.103.16 (RUSTSEC-2026-0382, distrusted intermediates could be accepted) and `webpki-roots` 1.0.11 (RUSTSEC-2026-0381, emSign intermediates); the lockfile of 1.1.1 still pinned the vulnerable versions.

## [1.1.1] — 2026-10-29

### Changed

- **Streamed audio uploads** (`#98`): meditation and soundscape audio is streamed to a scratch file under `$MEDIOSS_DATA_DIR/tmp` and validated, converted and probed from disk, so memory no longer grows with the file (release-mode peak RSS stays at about 40 MB for 20, 100 and 200 MB uploads; it was 77, 131 and 133 MB). Free space is checked up front and an upload is refused with `507` when it is below the configured limit; the `413` answer is unchanged. Keep free disk of at least twice the largest upload (staged input plus converted copy).

## [1.1.0] — 2026-10-29

### Added

- **Favorite meditations** (`#85`, `#86`, `#93`): a heart in the Player toggles a favorite; the Library's Favorites chip lists them newest-first as a full-width list naming every program the meditation belongs to. Favorites are per-user and purged with the user or meditation; data export/import does not carry them (known limit, documented in ARCHITECTURE.md).
- **Env-driven upload limits** (`#83`): `MEDIOSS_MAX_UPLOAD_BYTES`, `MEDIOSS_MAX_SOUND_UPLOAD_BYTES` and `MEDIOSS_MAX_COVER_BYTES` (defaults unchanged), validated at startup; README documents them (`#89`).
- **Favicon set** (`#81`): SVG/ICO favicons, Apple touch icon, web manifest and robots.txt.

### Fixed

- **Read-only container startup** (`#90`, `#91`): media scratch files now live in `$MEDIOSS_DATA_DIR/tmp` and `TMPDIR` points there, so the read-only-root Compose deployment can transcode and probe durations again (the published 1.0.0 image was broken for MP3/WAV/soundscape uploads); startup errors that occur before logging now print to stderr instead of exiting silently.
- **Early 413 for uploads** (`#84`): declared-oversize uploads are rejected before buffering, with multipart headroom for sound uploads.
- **Library search cards** (`#94`): search results keep their tile width instead of collapsing to ~50 px.
- **Sidebar and icon-rail scrolling** (`#92`, `#95`): in short windows the sidebar and the icon rail scroll on their own, rail tooltips live outside the scroll box (never clipped, no stray horizontal scrollbar) and survive a focus scroll.
- **Docker publish workflow** (`#78`, `#82`): publishes to ghcr.io with the built-in token; invalid env fixed, strict semver tag filter, `packages:write` scoped to the publish job.

### Performance

- **Duration probe** (`#88`): uploaded audio is staged straight to disk for the probe, removing two full in-memory copies (release-mode peak for a 100 MB upload: 243 MB → 156 MB).

### CI

- Lockfile and dependency guards (`#86`, `#87`): `cargo metadata --locked` check and an rsa-absent guard that fail on a stale lock.

## [1.0.0] — 2026-10-25

First tagged release. Everything below is relative to the pre-tag development history.

### Added

- **Custom date ranges** for History and Statistics: a shared popover with quick presets plus inclusive from/to local dates, validated on both the client and the backend (`from ≤ to`, at most 366 days).
- **Library state filters** — All / Completed / In progress / Not started — replacing category chips as the way to narrow the library.
- **Slider-style mood picker**: a never-wrapping five-stop track that fills to the chosen score, with a per-score expressive face (sad → nervous → meh → smiley → wink) in both picker sizes; keyboard and screen-reader behavior unchanged (native radios).
- **Program hero**: full-width cover with the scrim caption treatment shared by program cards, the player cover and the program page.
- **Active soundscape state**: the playing soundscape's button shows a filled, highlighted state.
- **About panel** in Settings over the new public `GET /api/meta`: app name, version (from the binary's Cargo version), source link, AGPL-3.0-or-later license, and the canonical donate link.
- **Per-module media roots** (`#30`): uploaded files now live under `media/meditation/`, `media/soundscape/`, and `media/identity/{user_id}/`; startup migrates a pre-0.5.0 flat `media/` layout once by renaming — no data loss, database keys unchanged.

### Changed

- **Soft-glass controls** (`#26`, `#28`): chips, text inputs, selects, textarea, toggle, checkbox, popover triggers and the mood stops use the glass-control treatment (translucent fill, rim, blur) with opaque fallbacks for reduced-transparency and no-backdrop-filter browsers; contrast tests composite the control fill over every surface at ≥ 4.5:1.
- **Demo covers** (`#24`): Unsplash photography for demo content with the unified overlay program card; cover scrim reaches full strength before the caption band so text holds 4.5:1 on any artwork.
- **CI hygiene** (`#29`): push builds run on `main` only, docs-only commits skip both workflows, pull-request Docker builds prove a single architecture, and concurrent runs on the same ref cancel their predecessor.

### Security

- `#23` split publishing out of every build: the Docker image publishes **only** on stable semver tags (`vX.Y.Z`, no prerelease), in one atomic push; `latest` always tracks the last stable release.
