---
title: Mood Tracker
id: 0006-mood-tracker
status: in_progress
capability: mood-tracker
created: 2026-09-29
plan: specs/0006-mood-tracker/PLAN.md
supersedes:
---

# Spec 0006 — Mood Tracker

## Intent

Every signed-in user can privately record how they feel with a mood score from 1 to 5 and an optional note. Users can review mood and note history, understand changes through summary statistics and charts, and follow meditation activity and mood records together in one chronological timeline. The capability supports personal reflection without exposing journal content or mood data to administrators or other users. It complements the existing meditation history and user statistics capabilities while preserving their current behavior.

## User Stories

### US1 — Record a mood entry (Priority: P1)

As a signed-in user, I want to record a 1–5 mood score and optionally describe how I feel, so I can capture my emotional state with or without writing a note.

**Independent test:** a signed-in user can save a score-only entry and an entry with a note, then see both in their own mood history; an invalid score is rejected without creating an entry.

- **Given** a signed-in user selects a mood score from 1 to 5, **when** they save without a note, **then** a timestamped mood entry is recorded.
- **Given** a signed-in user selects a mood score and writes a note, **when** they save, **then** the score and note are recorded together as one timestamped entry.
- **Given** a user submits a score outside 1–5, **when** the entry is validated, **then** it is rejected with a clear validation message and no entry is created.

### US2 — Review and manage mood history (Priority: P1)

As a signed-in user, I want to browse my previous mood scores and notes, so I can remember how I felt and what I wrote at the time.

**Independent test:** a user with several entries can see them newest first, open their complete notes, edit one entry, and delete one entry without affecting another user's data.

- **Given** a user has mood entries, **when** they open mood history, **then** their entries appear newest first with score, local date and time, and note when present.
- **Given** a user edits one of their entries, **when** they save valid changes, **then** the updated score and note replace the previous values while the original recorded time remains visible.
- **Given** a user confirms deletion of one of their entries, **when** deletion succeeds, **then** that entry disappears from history, analytics, charts, and the unified timeline.
- **Given** a user has no mood entries, **when** they open mood history, **then** they see an empty state that offers a direct way to record their first mood.

### US3 — Understand mood patterns (Priority: P1)

As a signed-in user, I want summary statistics and charts derived from my mood entries, so I can recognize patterns and changes over time.

**Independent test:** known mood entries produce the expected average, entry count, score distribution, and chronological trend for each supported date range.

- **Given** a user has mood entries in a selected range, **when** analytics are shown, **then** they see the entry count, average mood score, score distribution, and mood trend for that range.
- **Given** a local day contains multiple mood entries, **when** the trend is grouped by day, **then** that day displays the arithmetic mean of all mood scores recorded on that local day.
- **Given** a selected range has no mood entries, **when** analytics are shown, **then** charts and statistics display a clear no-data state rather than invented zero-valued mood data.

### US4 — Follow meditation and mood in one timeline (Priority: P1)

As a signed-in user, I want meditation activity and mood records in one timeline, so I can review my past practice, mood scores, and notes in chronological context.

**Independent test:** a user with completed meditation sessions, meditation notes, and mood entries sees all applicable records in one ordered timeline, while another user's records never appear.

- **Given** a user has completed meditation sessions, meditation notes, and mood entries, **when** they open the unified timeline, **then** all three event types appear together in reverse chronological order.
- **Given** a meditation note appears in the timeline, **when** it is rendered, **then** the note, its meditation context, and the note's own recorded time are presented as one meditation-note event.
- **Given** a mood entry has a note, **when** it appears in the timeline, **then** its mood score and note are presented as one mood event.
- **Given** multiple events have the same displayed local date, **when** the timeline is rendered, **then** their actual recorded timestamps determine a stable newest-first order.

### US5 — Keep personal reflection private (Priority: P1)

As a signed-in user, I want my mood data and personal notes to remain private, so I can use the tracker without exposing sensitive reflections.

**Independent test:** another member and an administrator cannot retrieve a user's mood entries, mood analytics, notes, or unified timeline content.

- **Given** a signed-in user requests mood or timeline data, **when** the response is produced, **then** it contains only records owned by that user.
- **Given** an administrator manages the instance, **when** they use administrative capabilities, **then** they cannot read members' mood scores, mood notes, meditation notes, or personal timeline content.
- **Given** a request is unauthenticated, **when** it targets mood or unified timeline behavior, **then** it is refused without disclosing personal data.

## Functional Requirements (EARS)

- **FR-001 (US1):** WHEN an authenticated user submits a mood score from 1 through 5 THE SYSTEM SHALL create a mood entry owned by that user with the submission timestamp.
- **FR-002 (US1):** WHEN an authenticated user includes a note with a valid mood score THE SYSTEM SHALL store the note as part of the same mood entry.
- **FR-003 (US1):** WHEN a mood score is missing, non-integral, or outside 1 through 5 THE SYSTEM SHALL reject the submission with a validation error and SHALL NOT create an entry.
- **FR-004 (US1):** WHEN an optional mood note contains only whitespace THE SYSTEM SHALL treat it as absent.
- **FR-005 (US1):** THE SYSTEM SHALL allow a user to create more than one mood entry on the same local day.
- **FR-006 (US1):** WHEN a mood note is submitted THE SYSTEM SHALL accept between 1 and 10,000 characters after trimming surrounding whitespace and SHALL reject longer notes without creating or changing an entry.
- **FR-007 (US2):** WHEN an authenticated user requests mood history THE SYSTEM SHALL return only that user's entries in reverse chronological order.
- **FR-008 (US2):** WHEN mood history is displayed THE SYSTEM SHALL show each entry's score, local date and time, and complete note when one exists.
- **FR-009 (US2):** WHEN an authenticated user saves valid changes to their own mood entry THE SYSTEM SHALL update its score and note without changing its original recorded timestamp.
- **FR-010 (US2):** WHEN an authenticated user confirms deletion of their own mood entry THE SYSTEM SHALL permanently remove it from history, analytics, charts, and the unified timeline.
- **FR-011 (US2, US5):** IF a user attempts to read, edit, or delete a mood entry owned by another user THEN THE SYSTEM SHALL refuse the operation without revealing that entry's contents.
- **FR-012 (US3):** WHEN an authenticated user requests mood analytics for a supported range THE SYSTEM SHALL compute all results exclusively from that user's mood entries within that range.
- **FR-013 (US3):** WHEN mood analytics contain at least one entry THE SYSTEM SHALL provide the entry count, arithmetic mean score, count and percentage for each score from 1 through 5, a score-distribution chart, and a chronological mood-trend chart.
- **FR-014 (US3):** WHEN multiple mood entries occur on one local day THE SYSTEM SHALL represent that day in the daily trend by the arithmetic mean of those entries.
- **FR-015 (US3):** WHEN a selected range contains no mood entries THE SYSTEM SHALL present a no-data state and SHALL NOT represent the absence of entries as a mood score of zero.
- **FR-016 (US3):** WHEN a date range is evaluated THE SYSTEM SHALL determine day boundaries using the user's request-local timezone offset rather than UTC day boundaries.
- **FR-017 (US3):** THE SYSTEM SHALL support mood analytics and history ranges of Today, Last 7 days, Last 30 days, and All time, each ending at the user's local current time.
- **FR-018 (US4):** WHEN an authenticated user requests the unified timeline THE SYSTEM SHALL combine that user's mood entries, meditation notes, and completed meditation sessions in reverse chronological order.
- **FR-019 (US4):** WHEN a meditation note appears in the unified timeline THE SYSTEM SHALL show the note text, its meditation context, and the note's own recorded timestamp as one meditation-note event.
- **FR-020 (US4):** WHEN a mood entry has a note THE SYSTEM SHALL show its score and note as one mood timeline event.
- **FR-021 (US4):** WHEN timeline events share the same displayed local date THE SYSTEM SHALL order them by their recorded timestamps, with a deterministic tie-breaker for equal timestamps.
- **FR-022 (US4):** WHEN the unified timeline range changes THE SYSTEM SHALL apply the same selected range to mood entries, meditation notes, and completed meditation sessions.
- **FR-023 (US5):** WHEN mood history, mood analytics, or unified timeline data is requested THE SYSTEM SHALL return data owned only by the authenticated user.
- **FR-024 (US5):** WHEN an administrator uses administrative capabilities THE SYSTEM SHALL NOT expose another user's mood scores, mood notes, meditation notes, or personal timeline content.
- **FR-025 (US5):** WHEN an unauthenticated request targets mood history, mood analytics, mood-entry mutation, or the unified timeline THE SYSTEM SHALL refuse the request.
- **FR-026 (US2, US3, US4):** WHEN an entry is created, edited, or deleted THE SYSTEM SHALL make the resulting history, analytics, charts, and timeline mutually consistent on the next successful view.
- **FR-027 (US2, US4):** WHEN history or timeline contains more records than one view can present accessibly THE SYSTEM SHALL let the user continue through the remaining records without silently omitting them.

## Success Criteria

- **SC-001:** A user can save a score-only entry and a score-with-note entry; both appear in mood history with the correct score and local timestamp.
- **SC-002:** Every attempted score below 1, above 5, fractional, or missing is rejected, and the user's entry count does not change.
- **SC-003:** For entries scored 1, 3, and 5 in one selected range, analytics report `3` entries, an average of `3.0`, and one entry in each corresponding distribution bucket.
- **SC-004:** For scores 2 and 4 recorded on the same local day, the daily trend reports one point for that day with an average of `3.0`.
- **SC-005:** Editing an entry changes its value in history, analytics, charts, and timeline while preserving its original recorded timestamp.
- **SC-006:** Deleting an entry removes it from history, analytics, charts, and timeline on the next successful view.
- **SC-007:** A unified timeline containing two mood entries, two meditation notes, and two completed meditation sessions displays exactly six events in deterministic reverse chronological order.
- **SC-008:** A mood entry and a completed meditation session that fall on different UTC dates but the same request-local date are grouped under that same local date.
- **SC-009:** Requests made as Member A never return Member B's mood score, mood note, meditation note, analytics contribution, or timeline event; administrative access provides no exception.
- **SC-010:** A first-time user with no records sees actionable empty states for mood history, analytics, and the unified timeline without errors or fabricated statistics.
- **SC-011:** All mood entry, history, analytics, and timeline controls and visualizations are usable at mobile and desktop widths and meet the project's WCAG 2.1 AA target.
- **SC-012:** History and timeline navigation expose every matching record in a test set larger than one view's display capacity, with no duplicates or omissions.

## Edge Cases

- The user records several different moods within minutes or on the same local day.
- The user's timezone offset causes an entry's local date to differ from its UTC date.
- The selected date range includes a daylight-saving transition; each request uses its supplied local offset consistently.
- A mood note is absent, contains only whitespace, or contains multiple paragraphs and non-ASCII text.
- A user edits an old entry after newer entries exist; ordering continues to use the original recorded timestamp.
- A mood entry is deleted while it is visible in another open view; the next successful refresh no longer includes it.
- Mood or meditation records share an identical timestamp; ordering remains deterministic.
- The user has mood entries but no meditation sessions, meditation sessions but no mood entries, or neither.
- A completed meditation has no practice note; it still appears as a meditation event without invented note content.
- A range has records just inside and just outside its local-time boundary.

## Assumptions

- The requested 1–5 mood score supersedes the PRD's earlier 1–10 proposal for this capability; `1` represents the lowest mood and `5` the highest.
- Users may record multiple mood entries per day because emotional state can change during a day.
- A mood note is optional free-form personal text; emotion tags, activity tags, and mandatory prompts are not required for the first capability.
- Editing preserves the original recorded timestamp so users cannot accidentally rewrite chronology; the latest content is what history and analytics use.
- The initial statistics are entry count and arithmetic mean; the initial charts are a 1–5 score distribution and a chronological daily-average mood trend.
- Today, Last 7 days, Last 30 days, and All time are sufficient initial range presets and apply consistently to history, analytics, and timeline views.
- The unified timeline treats existing completed meditation sessions and existing meditation notes as separate event types because meditation notes belong to a meditation and carry their own timestamps; incomplete playback activity is not a timeline event.
- Mood and journal content is sensitive personal data and remains visible only to its owner, including on self-hosted instances administered by someone else.
- Dates and times are presented in the user's current request-local timezone; stored event chronology remains stable when the viewing timezone changes.
- Deletion is permanent from normal product behavior; backup retention and operator-level disaster recovery remain deployment concerns.

## Out of Scope

- Emotion labels, activity tags, symptom tracking, diagnoses, clinical recommendations, or crisis intervention.
- Reminders, notifications, scheduled check-ins, or prompts after meditation and soundscape sessions.
- Automated correlation, causal claims, or recommendations connecting meditation activity to mood scores.
- Mood forecasting, sentiment analysis of notes, or AI-generated interpretations.
- Sharing mood entries, notes, analytics, or timelines with administrators, other users, clinicians, or external services.
- Import, export, backup, and cross-instance migration of mood records.
- Offline creation and later synchronization of mood entries.
- Soundscape playback events in the unified timeline.
- Changes to existing meditation recording, meditation statistics, or meditation-note semantics.

## Clarifications

No open clarifications.
