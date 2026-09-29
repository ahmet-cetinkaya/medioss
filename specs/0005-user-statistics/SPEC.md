---
title: User Statistics
id: 0005-user-statistics
status: shipped
capability: user-statistics
created: 2026-09-28
source-of-truth: meditation_sessions (completed session records)
---

# Spec 0005 — User Statistics

## Intent

Every signed-in user can see a summary of their own meditation practice: total minutes, total days, current streak, longest streak, and a calendar marking the days they meditated. The summary and calendar filter across three preset ranges: **Today**, **Last 30 days**, **All time**.

The data source is the existing completed-session records (rows written by `record_completion`); no new recording mechanism is added.

## User Stories

- **US1 (P1)** — As a signed-in user, I want total minutes, total days, current streak, and longest streak at a glance, so I can understand the volume and continuity of my practice.
- **US2 (P1)** — As a user, I want to see which days I meditated on a calendar, so I can visually recall my rhythm.
- **US3 (P2)** — As a user, I want to filter the summary and calendar by Today / Last 30 days / All time, so I can view my daily progress separately from long-term trends.

## Functional Requirements (EARS)

- **FR-001**: WHEN an authenticated user requests statistics THE SYSTEM SHALL return values computed only from that user's own completed sessions; another user's data shall not appear in any field.
- **FR-002**: THE SYSTEM SHALL provide four summary metrics: **total minutes**, **total days** (count of distinct local days containing at least one completed session), **current streak**, **longest streak**.
- **FR-003**: WHEN a range is selected (Today / Last 30 days / All time) THE SYSTEM SHALL compute total minutes, total days, and calendar days for that range; current and longest streak shall always be computed over all time (streak metrics are not filtered).
- **FR-004**: WHEN calendar data is returned THE SYSTEM SHALL list every meditated local day in the selected range as a local date (`YYYY-MM-DD`) with that day's total minutes.
- **FR-005**: THE SYSTEM SHALL determine day boundaries by the local timezone offset (in minutes) sent with the request; it shall not use the UTC day boundary.
- **FR-006**: WHEN the request is unauthenticated THE SYSTEM SHALL return `401`.
- **FR-007**: WHEN the request carries an invalid offset or an undefined range value THE SYSTEM SHALL return `400` with a validation error.
- **FR-008**: THE SYSTEM SHALL define ranges relative to the user's local today: Today = the local current day only; Last 30 days = the 30 days ending with local today inclusive; All time = unbounded.

## Success Criteria

- **SC-001**: A user with two completed sessions of 25 and 35 minutes on the same local day sees, under All time, total minutes `60`, total days `1`, and one calendar day.
- **SC-002**: A user with sessions on each of the last 5 local days but none yet today has a current streak of `5` that does not break before the local day ends; the longest streak is at least `5`.
- **SC-003**: The Last-30-days filter excludes a session 31 days ago from totals and calendar; streaks are still computed over all time.
- **SC-004**: Member A's summary never contains member B's session minutes or days.
- **SC-005**: An unauthenticated request receives `401`; every authenticated user can view their own statistics regardless of role (no admin privilege).
- **SC-006**: A user with no sessions receives zero summaries and an empty calendar, not an error.

## Edge Cases

- **Session crossing midnight**: the session counts toward the local day of `started_at`; its duration (finish − start) is added to totals in full.
- **DST offset change**: day boundaries use the current offset sent with the request; separate historical offsets are not accepted.
- **Multiple overlapping sessions**: each completed session counts with its own duration; overlaps are not trimmed (source records are idempotent: user + meditation + start time is unique).
- **Zero-duration session** (start = finish): counts toward the day count, adds `0` minutes.

## Assumptions

- **A1 — Source**: `meditation_sessions` is written only by the `record_completion` flow; statistics measure completed practice, not abandoned listens.
- **A2 — Total minutes**: the sum of completed sessions' `finished_at − started_at`, floored once to whole minutes.
- **A3 — Streak definition**: a chain of consecutive local days each containing at least one completed session; if there is no session today, the chain counts backward from yesterday and is not considered broken until the local day ends.
- **A4 — "Daily" filter** is interpreted as Today (a single local day).
- **A5 — Offset transmission**: the client sends its current offset in minutes (e.g. `-new Date().getTimezoneOffset()`) with every request; the server does not store it.
- **A6 — Placement**: which module/crate owns the statistics and how it accesses the session table is an architecture decision; this spec defines behavior only.

## Out of Scope

- Other users' statistics, comparisons, or leaderboards.
- Per-program or per-meditation breakdowns (most-listened, etc.).
- Custom weekly/monthly ranges and picking ranges from the calendar (three preset ranges only).
- Soundscape (mixer) usage statistics.
- Statistics export, heatmap color schemes, and similar visual details (UI layout belongs to the plan).
- Retrospective data migration: existing session rows are used as-is.

## Clarifications

(none)
