---
title: Data Portability
id: 0008-data-portability
status: shipped
capability: data-portability
created: 2026-10-03
plan: specs/changes/0008-data-portability/PLAN.md
supersedes:
---

# Spec 0008 — Data Portability

## Intent

Every signed-in user can take their own data with them: download it in standard formats (JSON, CSV) and import previously exported files into another instance, so a self-hosted migration or personal backup restore is a user-controlled operation with a lossless round-trip. The capability covers the caller's own records only — mood entries first, meditation practice data second — and never exposes one user's data to another or to administrators beyond what existing capabilities already allow. It fulfills the PRD's portability requirements (docs/PRD.md lines 229–230, 292, 447) and closes roadmap milestone M7's export/import deliverable; the weekly/monthly mood-trend charts also listed under M7 are already delivered by spec 0006's rolling ranges (Last7Days, Last30Days).

## User Stories

### US1 — Export my data (Priority: P1)

As a signed-in user, I want to download my own data in standard formats, so I can keep a personal backup or prepare a migration to another instance.

**Independent test:** a user with mood entries and meditation practice data downloads a JSON file and a CSV file; both contain only their own records, and another user's or the admin-curated library's records never appear.

- **Given** a user has mood entries, **when** they export, **then** they receive a JSON document containing every mood entry with its score, note, and original recorded time, and a CSV file with the same mood entries in spreadsheet-friendly columns.
- **Given** a user has meditation practice data (sessions, notes, playback positions), **when** they export, **then** the JSON document also contains those records, each referencing its meditation by name and duration rather than by an instance-local identifier.
- **Given** a user has no data of a kind, **when** they export, **then** the corresponding section is present but empty rather than omitted, so the format is stable.
- **Given** a user exports, **when** the files are produced, **then** they carry a format version so future instances can import old exports.

### US2 — Import mood data with lossless round-trip (Priority: P1)

As a signed-in user, I want to import a previously exported mood file into an instance, so that my mood history, analytics, charts, and timeline are reproduced exactly as they were.

**Independent test:** export from one instance, import into a fresh instance, and the mood history, statistics, trend charts, and unified timeline show the same entries with the same scores, notes, and recorded times as the origin.

- **Given** a valid JSON export, **when** the user imports it, **then** every mood entry is recreated with its original score, note, and recorded time.
- **Given** a valid CSV export of mood entries, **when** the user imports it, **then** every mood entry is recreated with its original score, note, and recorded time.
- **Given** an import file whose mood entries exactly duplicate entries the user already has (same recorded time, score, and note), **when** the user imports it, **then** those duplicates are skipped rather than recorded twice.
- **Given** an import file that is malformed, truncated, or not a recognized export format, **when** the user imports it, **then** the import is rejected with a clear message and no entry is created.

### US3 — Import meditation practice data (Priority: P2)

As a signed-in user, I want my meditation practice data (sessions, notes, playback positions) restored alongside my mood data, so a migrated instance reflects my whole practice history.

**Independent test:** on a fresh instance whose admin has re-created the same library meditations (same names and durations), importing a JSON export restores sessions, notes, and playback positions to the matching meditations.

- **Given** a JSON export containing practice data and a target instance whose library contains a meditation with the same name and duration, **when** the import runs, **then** sessions, notes, and the playback position for that meditation are attached to the target instance's meditation.
- **Given** an exported practice record references a meditation that does not exist on the target instance (by name and duration), **when** the import runs, **then** that record is skipped and reported, and the rest of the import still completes.
- **Given** practice records exist in the export, **when** the import completes, **then** the user is shown how many practice records were linked, skipped, or already present.

### US4 — Trustworthy import feedback (Priority: P1)

As a signed-in user, I want the import to tell me what happened and never leave my data half-written from a broken file, so I can trust the operation.

**Independent test:** importing a file with one invalid record creates nothing and reports the problem; importing a valid file reports counts of what was created, linked, skipped as duplicate, and skipped as unmatchable.

- **Given** any import file, **when** parsing or validation fails, **then** nothing is written and the user sees a clear error identifying the problem.
- **Given** a successful import, **when** it finishes, **then** the user sees per-kind counts (mood entries created, duplicates skipped, practice records linked, practice records skipped as unmatchable).
- **Given** an import completes, **when** the user views history, analytics, or the timeline, **then** all views reflect the imported data consistently, as if the entries had been recorded natively.

## Functional Requirements

Format and export behavior:

- **FR-001 (US1):** WHEN an authenticated user requests a full export THE SYSTEM SHALL produce a single JSON document, UTF-8 encoded, containing a format version, the exporting user's profile snapshot (display name only, for information), and complete sections for mood entries and meditation practice data, each record carrying its original recorded or completed time.
- **FR-002 (US1):** WHEN an authenticated user requests a CSV export THE SYSTEM SHALL produce a UTF-8 CSV document of mood entries with stable columns (recorded time, score, note) sufficient to reproduce those entries on import.
- **FR-003 (US1):** WHEN an export is produced THE SYSTEM SHALL reference meditations in practice data by name and duration, not by instance-local identifiers, and SHALL NOT include audio blobs, cover images, program or category definitions, or any other user's records.
- **FR-004 (US1):** WHEN a user exports a kind of data they have none of THE SYSTEM SHALL emit the corresponding section as empty rather than omitting it.

Import behavior:

- **FR-005 (US2):** WHEN an authenticated user imports a valid JSON export THE SYSTEM SHALL recreate every mood entry with its original score, note, and recorded time, applying duplicate detection per FR-007.
- **FR-006 (US2):** WHEN an authenticated user imports a valid mood CSV THE SYSTEM SHALL recreate every mood entry with its original score, note, and recorded time, applying duplicate detection per FR-007.
- **FR-007 (US2, US4):** WHEN an imported mood entry has the same recorded time, score, and note as an entry the importing user already has THE SYSTEM SHALL skip it instead of creating a second copy.
- **FR-008 (US4):** WHEN an import file is malformed, uses an unknown format version, or contains a record that fails validation THE SYSTEM SHALL reject the entire import with a clear message and write nothing.
- **FR-009 (US3):** WHEN an imported practice record's meditation (by name and duration) exists on the target instance THE SYSTEM SHALL attach the record to that meditation.
- **FR-010 (US3):** WHEN an imported practice record's meditation does not exist on the target instance THE SYSTEM SHALL skip that record, continue the import, and include it in the skipped count.
- **FR-011 (US4):** WHEN an import succeeds THE SYSTEM SHALL report per-kind counts: mood entries created, mood duplicates skipped, practice records linked, practice records skipped as unmatchable, and practice records skipped as duplicates of records the user already has.
- **FR-012 (US4):** WHEN imported data is written THE SYSTEM SHALL leave history, analytics, charts, and the unified timeline consistent with natively recorded data on the next successful view.

Boundaries and privacy:

- **FR-013 (US1–US4):** WHEN export or import is performed THE SYSTEM SHALL restrict all read and write access to the authenticated caller's own records; an administrator SHALL NOT be able to export or import another user's data through this capability.
- **FR-014 (US2–US4):** WHEN an import completes THE SYSTEM SHALL NOT modify the importing user's identity fields (email, password, role) or any other user's data.

## Success Criteria

- **SC-001:** On an instance with mood entries, meditation practice data, and imported-file edge cases prepared, exporting to JSON and importing into a fresh instance reproduces the mood history exactly: same count, same scores, notes, and recorded times, and identical statistics (count, mean, distribution) for every range offered by analytics.
- **SC-002:** A CSV export imported into a fresh instance reproduces the same mood history as the JSON round-trip in SC-001.
- **SC-003:** Importing the same file twice creates the second import's mood entries zero times (all detected as duplicates) and reports them as skipped.
- **SC-004:** On a fresh instance whose library re-creates meditations with the same names and durations, importing a JSON export links sessions, notes, and playback positions to the correct meditations; removing one library meditation before import causes exactly that meditation's practice records to be reported as skipped with the rest imported.
- **SC-005:** A malformed or unknown-version file is rejected with nothing written, verifiable by unchanged history after the attempt.
- **SC-006:** A second user (or administrator) cannot retrieve or alter another user's data through any export or import operation.

## Edge Cases

- Notes containing commas, quotes, newlines, and multi-byte characters must survive the CSV round-trip.
- An export produced by a future format version is rejected clearly, not partially interpreted.
- Empty sections (no mood entries, no practice data) import as no-ops with zero counts, not as errors.
- Two distinct mood entries sharing the same recorded time but differing in score or note are both imported (duplicate detection is on the full triple).
- Practice data for a meditation that exists with the same name but a different duration is treated as unmatchable and skipped with a report.
- Import of a file whose recorded times fall outside any expected range is accepted as-is; recorded times are authoritative from the export.

## Assumptions

- Weekly/monthly mood charts in M7 are satisfied by spec 0006's shipped rolling ranges (Last7Days, Last30Days); this change adds no chart work.
- JSON is the canonical round-trip format; CSV is a mood-only view that also round-trips because mood columns are complete. Practice data never appears in CSV.
- Practice data means meditation sessions, practice notes, and playback positions (completed state included). Program membership progress is derived from sessions and is not exported separately.
- Duplicate detection for mood entries uses the exact triple (recorded time, score, note); no fuzzy matching.
- Import is atomic per file: full parse and validation precede any write.
- Import appends; it never deletes or overwrites existing entries (a "replace all" mode is out of scope for v1).
- Identity fields in the export are informational only and are never applied on import.

## Out of Scope

- Exporting or importing the admin-curated shared library (audio, covers, programs, categories) — instance operators migrate that via data-directory backup (M9 backup/restore work).
- Bulk/administrative export of all users' data.
- Cloud sync, scheduled automatic backups, differential or merge imports.
- Deleting data (covered by the existing self-deletion flow in spec 0004 / profile settings).
- Any change to charts or analytics rendering.

## Preserved Behavior

- **PRESERVED:** Spec 0006's mood behavior is unchanged: history ordering, edit and delete semantics, analytics ranges, charts, and the unified timeline keep working exactly as shipped for natively recorded entries.
- **PRESERVED:** Spec 0007's profile settings keep their shipped behavior; this capability adds a data section alongside them without altering display name or avatar flows.
- **PRESERVED:** Privacy boundaries from specs 0004/0006 hold: mood notes and practice data remain invisible to administrators and other users, including through the new export surface.

## Deviations

- Review fix (2026-10-04, PR #11 review M3): ImportReport gained `practice_duplicates_skipped` and repeat imports skip practice records that already exist, extending SC-003's no-inflation guarantee from mood-only to practice data. FR-011 amended accordingly.

## Clarifications

- Q: Approve spec 0008? → A: User approved by advancing directly to `/ac:plan` on this spec (2026-10-03).
