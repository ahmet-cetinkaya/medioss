# Changelog

All notable changes to MediOSS are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning is semantic (`MAJOR.MINOR.PATCH`), and only stable tags publish a Docker image (`latest` tracks the last stable release).

## [1.0.0] — unreleased

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
