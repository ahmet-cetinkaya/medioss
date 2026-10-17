---
title: UI/UX Design
id: 0009-ui-ux-design
status: shipped
capability: web-experience
created: 2026-10-04
supersedes:
---

# Spec 0009 — UI/UX Design

## Intent

Every MediOSS web screen is redesigned to the "Soft Glass" design language so the app feels calm, coherent, and navigable instead of a set of disconnected pages. Today there is no global navigation, no way to sign out, inconsistent handling of signed-out visitors, blank screens while data loads, browser-native dialogs, and several visible defects (a search result that leads nowhere, an Admin entry shown to everyone, mix volumes that ignore what was saved, specific error messages that never appear). After this change, signed-in users move between all features through one app shell, every screen communicates loading, empty, and error states, the whole interface can be switched between English and Turkish, and the experience is usable on a 320 px phone and on a wide desktop while meeting WCAG 2.1 AA and making no third-party requests. The visual source of truth is the Soft Glass design system (`docs/DESIGN_SYSTEM.md`, called "the design system" below); this spec states what each screen must show and do, and which existing behavior must survive the redesign.

## User Stories

### US1 — Move through the app from one shell (Priority: P1)

As a signed-in user, I want the same navigation, page title, and account controls on every screen, and consistent feedback while things load or fail, so I always know where I am and can reach any feature or sign out.

**Independent test:** sign in, visit every navigation destination from every other one using only the navigation, sign out, and confirm that opening any authenticated screen address afterwards shows the login screen; force a not-found and a network failure on Player and Admin and confirm an error state appears instead of the login screen.

- **Given** a signed-in member on Library, **when** they look at the navigation, **then** they see the Practice, Progress, and Account groups, Library is the only item marked as current, and no Admin item exists.
- **Given** a signed-in administrator on any authenticated screen, **when** they look at the navigation, **then** an Admin item is present in the Account group.
- **Given** a signed-in user, **when** they choose Sign out and confirm in the in-app dialog, **then** their session ends and the login screen is shown; **when** they cancel instead, **then** nothing changes.
- **Given** a signed-out visitor, **when** they open the address of any authenticated screen (for example a Player address), **then** the login screen is shown, and after signing in they land on the screen they originally requested.
- **Given** a signed-in user opens a Player address for a meditation that does not exist, **when** the screen loads, **then** a not-found state with a way back to Library is shown and the login screen is not shown.
- **Given** a slow connection, **when** any authenticated screen is opening, **then** every region waiting for data shows a loading state and no screen is blank.
- **Given** a user deletes a soundscape mix, **when** they choose delete, **then** an in-app confirmation dialog naming the mix appears, focus is inside it, and Escape cancels it.

### US2 — Start the day from Home (Priority: P1)

As a signed-in user, I want Home to greet me and put my next meditation, today's suggestion, and my recent programs one step away, so I can start practicing without searching.

**Independent test:** with a user who has a program in progress, a daily suggestion, and recent programs, open Home and start each from its pill or card; with a brand-new user on an empty instance, open Home and see an empty state with a next action in every region.

- **Given** the device's local time is 08:00 and the user's display name is "Ayşe", **when** Home opens, **then** the greeting reads as a morning greeting that includes "Ayşe".
- **Given** the user has no display name, **when** Home opens, **then** the greeting contains no name and never shows the email address.
- **Given** Home is displayed, **when** the user looks at the destination tiles, **then** exactly one tile (Library) is featured and Soundscape, Mood, History, and Statistics tiles are present.
- **Given** the user has a program in progress, **when** they press play on the continue-program pill, **then** the Player opens on that program's next meditation.
- **Given** a brand-new user on an instance without content, **when** Home opens, **then** each region (continue program, daily suggestion, recent programs) shows an empty state and no region is blank.
- **Given** the daily-suggestion request fails, **when** Home opens, **then** only that region shows an error with a retry action and the rest of Home stays usable.

### US3 — Practice: Library, Program, Player, Soundscape (Priority: P1)

As a signed-in user, I want to browse and search the library, open a program, listen to a meditation, and mix soundscapes on screens that show the right content and keep my saved settings, so practicing is effortless.

**Independent test:** search the library and open a result in the Player; open a program, see its cover and start it; in the Player see the meditation name, play, seek, change speed, add and delete a note; in Soundscape load a saved mix and see its saved volumes, then delete it with confirmation.

- **Given** a library search returns a meditation, **when** the user opens that result, **then** the Player for that meditation opens.
- **Given** a program with an uploaded cover, **when** its Program screen opens, **then** the cover is shown as the hero artwork.
- **Given** a meditation named "Body Scan", **when** its Player opens, **then** "Body Scan" is the page title.
- **Given** a saved mix where Rain is at 30 and Wind at 70, **when** the user loads it, **then** the Rain control shows 30 and the Wind control shows 70.
- **Given** a meditation note, **when** the user chooses delete, **then** a confirmation dialog appears and cancelling keeps the note.
- **Given** a member and an empty library, **when** they open Library, **then** they see an empty state saying the host has not added programs yet and no Admin link.
- **Given** a 320 px wide screen, **when** the user opens the Player, **then** every control is reachable without horizontal page scrolling.

### US4 — Review progress: Mood, History, Statistics (Priority: P2)

As a signed-in user, I want my mood tracker, timeline, and statistics presented as calm panels with clear filters, so I can reflect on my practice and mood at a glance.

**Independent test:** record, edit, and delete a mood entry; switch ranges on Mood, History, and Statistics; filter History to notes only and load more; confirm empty states for a user without records.

- **Given** the Mood screen, **when** the user picks score 4 and saves a note, **then** the entry appears in the history list with score 4, local date and time, and the note.
- **Given** the History screen with more records than one page, **when** the user chooses load more, **then** further records are appended with no duplicates.
- **Given** a user practiced at 23:30 local time on a day whose UTC date is the next day, **when** Statistics lists practiced days, **then** that day appears under the local date.
- **Given** a user with no mood entries in the selected range, **when** Mood is shown, **then** a no-data state replaces the chart instead of a zero-valued chart.
- **Given** a signed-out visitor opens Mood, History, or Statistics, **when** the screen loads, **then** the login screen is shown instead of error text.

### US5 — Manage account and instance: Settings, Admin, Login, Setup (Priority: P2)

As a user or administrator, I want account, administration, sign-in, and first-run setup screens that follow the same design and show precise feedback, so account tasks are safe and understandable.

**Independent test:** sign in (including a throttled attempt), change display name and avatar, export and import data, try to delete the last admin account and read the specific refusal; as admin, create a user, disable and re-enable them, rename a sound choosing its category by name; on a fresh instance, complete setup.

- **Given** the administrator creates a user with an email that already exists, **when** the request is refused, **then** the form keeps its values and shows the duplicate-email message.
- **Given** the only active administrator tries to delete their own account from Settings, **when** the request is refused, **then** the specific last-active-administrator message is shown.
- **Given** an administrator chooses Disable for a member, **when** the confirmation dialog opens, **then** it states the member will be signed out immediately.
- **Given** a member opens the Admin address, **when** the screen loads, **then** an admins-only state with a link to Home is shown and no admin data is requested.
- **Given** a login attempt is throttled, **when** the refusal arrives, **then** a countdown is shown and the sign-in action stays disabled until it reaches zero.
- **Given** the Setup screen is opening and the setup status is not yet known, **when** the visitor tries to fill in or submit the setup form, **then** the form is not operable.

### US6 — Accessible, responsive, private by default (Priority: P1)

As any user, including keyboard, screen-reader, low-vision, and phone users, I want every screen to be operable and legible at any width and with my motion and transparency preferences, without my browser contacting third parties, so the app is usable and trustworthy.

**Independent test:** run an automated accessibility audit and a keyboard-only walkthrough on every screen; view every screen at 320, 768, 1024, and 1440 px; enable reduced motion and reduced transparency; record network traffic during a full walkthrough and confirm every request targets the instance.

- **Given** a screen narrower than 768 px, **when** any authenticated screen is shown, **then** navigation appears as a bottom bar and no page-level horizontal scrolling is possible.
- **Given** a 1024–1279 px wide screen, **when** an authenticated screen is shown, **then** navigation is an icon rail whose items still expose their names to assistive technology and on focus or hover.
- **Given** the user has asked the operating system to reduce transparency, **when** any screen is shown, **then** all surfaces are opaque.
- **Given** a keyboard-only user, **when** they tab through any screen, **then** every interactive element shows a visible focus indicator and every action can be completed.
- **Given** a full walkthrough of all screens, **when** network traffic is recorded, **then** zero requests go to an origin other than the instance.

### US7 — Change my password, and change a temporary one before anything else (Priority: P2)

As a user, I want to change my password from Settings, and as a user created with a temporary password I want to be asked to replace it right after signing in, so my account is protected by a password only I know.

**Independent test:** an administrator creates a member; the member signs in with the temporary password, is shown the change-password screen, cannot reach other screens until they change it, then lands on Home; the member later changes the password again from Settings and signs in with the new one.

- **Given** a user flagged to change their password, **when** they sign in, **then** the change-password screen is shown without navigation.
- **Given** that user tries to open Library by address, **when** the screen loads, **then** the change-password screen is shown instead.
- **Given** the user enters a 7-character password, **when** they submit, **then** the minimum-length rule is shown and nothing changes.
- **Given** the two entered passwords differ, **when** they submit, **then** a mismatch message is shown and no request is made.
- **Given** a valid new password, **when** the change succeeds, **then** the user reaches Home (or the screen they originally requested) and stays signed in.

### US8 — Check in my mood from Home (Priority: P2)

As a signed-in user, I want to record a quick mood score and optional note directly on Home and see my latest entries there, so checking in takes seconds.

**Independent test:** from Home, save a score-only check-in and a score-with-note check-in; both appear in Home's recent entries and on the Mood screen with the same score, note, and time.

- **Given** Home, **when** the user selects score 2, types a note, and saves, **then** a confirmation is announced, the form resets, and the entry is listed among Home's recent mood entries and on Mood.
- **Given** a user without mood entries, **when** Home opens, **then** the recent-mood region shows an empty state inviting a first check-in.
- **Given** saving the check-in fails, **when** the error arrives, **then** the selected score and note are kept and the reason is shown.

### US9 — Use the app in my language (Priority: P1)

As any visitor or user, I want to choose whether the interface is in English or Turkish, and have it remembered, so I can use MediOSS in the language I am comfortable with.

**Independent test:** on a browser that prefers Turkish, open Login and see Turkish; switch to English on Login and sign in; in Settings switch back to Turkish and see every screen and dialog in Turkish; reload, sign out, and sign in again and confirm Turkish is kept.

- **Given** a first visit from a browser whose preferred languages list Turkish before English, **when** Login opens, **then** the interface is in Turkish.
- **Given** a first visit from a browser that prefers neither Turkish nor English, **when** any screen opens, **then** the interface is in English.
- **Given** a signed-in user on Settings, **when** they choose English, **then** all interface text on the current screen changes to English without a sign-out and without losing other unsaved input on the screen.
- **Given** a user chose Turkish, **when** they reload, sign out, and sign in again on the same device, **then** the interface stays in Turkish.
- **Given** the interface is in Turkish, **when** a meditation date and duration are shown, **then** they use Turkish date, time, and number formatting.
- **Given** the interface is in Turkish and the server refuses an action with a recognized reason (for example duplicate email), **when** the refusal arrives, **then** the reason is shown in Turkish.

## Requirements

### ADDED

#### App shell, navigation, and shared states (US1)

- **FR-001 (US1):** THE SYSTEM SHALL present each authenticated screen (Home, Library, Program, Player, Soundscape, Mood, History, Statistics, Settings, Admin) inside one shared app shell containing global navigation, the brand lockup, the screen's page title, and an account button, following design system sections 4.1 and 5.1–5.4.
- **FR-002 (US1):** THE SYSTEM SHALL arrange global navigation into three labeled groups in this order: Practice (Home, Library, Soundscape, Mood), Progress (History, Statistics), and Account (Settings, Admin, Sign out).
- **FR-003 (US1):** WHERE the signed-in user is not an administrator THE SYSTEM SHALL omit the Admin navigation item and every other link, button, or tile leading to Admin from every screen.
- **FR-004 (US1):** WHEN an authenticated screen is displayed THE SYSTEM SHALL mark exactly one item in each visible navigation surface as current, both visually (through weight and icon change, not color alone) and to assistive technology, where Program and Player mark Library as current.
- **FR-005 (US1):** THE SYSTEM SHALL show each screen's name as its page title, as the screen's only top-level heading, and in the browser tab title combined with the product name.
- **FR-006 (US1):** WHEN the user activates the account button THE SYSTEM SHALL open Settings, where the button carries an accessible name and shows the user's avatar if one exists, otherwise the first letter of the display name uppercased with Turkish casing rules as today ("i" becomes "İ"), otherwise a generic person icon.
- **FR-007 (US1):** WHEN the user activates Sign out THE SYSTEM SHALL ask for confirmation in an in-app dialog, and on confirmation end the session and show the login screen.
- **FR-008 (US1):** IF ending the session fails for any reason other than the session having already ended THEN THE SYSTEM SHALL keep the user on the current screen and show a message that sign-out did not complete.
- **FR-009 (US1):** WHEN a signed-out visitor opens an authenticated screen, or any request made by an authenticated screen is refused because the session is no longer valid, THE SYSTEM SHALL show the login screen and remember the requested screen address.
- **FR-010 (US1):** WHEN sign-in succeeds THE SYSTEM SHALL open the remembered screen address with its original parameters, or Home when no address is remembered or the remembered address is not one of this application's screens.
- **FR-011 (US1):** IF loading a screen or region fails for a reason other than an invalid session (not found, network failure, server error) THEN THE SYSTEM SHALL show an error state in the affected region that states the problem and offers a retry action, and SHALL NOT show the login screen.
- **FR-012 (US1):** IF the item a Program or Player address refers to does not exist THEN THE SYSTEM SHALL show a not-found state with a link to Library.
- **FR-013 (US1):** WHILE a region is waiting for its data THE SYSTEM SHALL show a loading state in that region that reserves its space, so no screen or region renders blank.
- **FR-014 (US1):** WHEN a region has no data to show THE SYSTEM SHALL show an empty state that explains what will appear there and offers the next relevant action where one exists.
- **FR-015 (US1):** WHEN an action is refused and the server supplies a specific reason (for example last active administrator, duplicate email, file too large, category still contains sounds, minimum password length) THE SYSTEM SHALL show that specific reason next to the action instead of a generic failure message.
- **FR-016 (US1):** WHEN an action needs confirmation or a short text input (destructive actions, sign-out, renames) THE SYSTEM SHALL use an in-app dialog styled per the design system and SHALL NOT use browser-native confirm, prompt, or alert dialogs.
- **FR-017 (US1):** WHILE an in-app dialog is open THE SYSTEM SHALL place keyboard focus inside it, keep focus within it, treat Escape as cancel, and return focus to the control that opened it when it closes.
- **FR-018 (US1):** WHEN a confirmation dialog for a destructive action opens THE SYSTEM SHALL name the affected item and the consequence, label the confirming action with a specific verb (for example "Delete mix"), and place initial focus on the cancel action.
- **FR-019 (US1):** WHEN a rename dialog opens THE SYSTEM SHALL prefill the current name and SHALL NOT allow saving an empty or whitespace-only name.
- **FR-020 (US1):** WHILE an action is being submitted THE SYSTEM SHALL show progress on the triggering control and prevent the same action from being submitted again until it finishes.
- **FR-021 (US1):** WHEN an action completes or fails without leaving the screen THE SYSTEM SHALL announce the outcome message to assistive technology without moving focus.

#### Home (US2)

- **FR-022 (US2):** WHEN Home is displayed THE SYSTEM SHALL show a greeting hero (design system section 5.8) whose greeting is chosen by the device's local time: morning from 05:00 to 11:59, afternoon from 12:00 to 17:59, evening from 18:00 to 04:59.
- **FR-023 (US2):** WHERE the user has a display name THE SYSTEM SHALL include it in the greeting, and otherwise SHALL show the greeting without any name and without the email address.
- **FR-024 (US2):** THE SYSTEM SHALL show Home destination tiles with Library as the only featured tile and Soundscape, Mood, History, and Statistics as standard tiles, each tile being a single link to its screen.
- **FR-025 (US2):** WHERE the user has a program in progress THE SYSTEM SHALL show a continue-program pill with the program title and next meditation, whose play action opens the Player on that next meditation and whose title opens the Program screen.
- **FR-026 (US2):** WHERE a daily suggestion exists THE SYSTEM SHALL show it as a suggestion pill with the meditation name and duration whose play action opens the Player for that meditation.
- **FR-027 (US2):** WHERE the user has recently experienced programs THE SYSTEM SHALL show up to five of them as content cards with artwork, title, and progress label, each opening its Program screen, plus a "See all" link to History.
- **FR-028 (US2):** IF the user has no program in progress, the instance offers no daily suggestion, or the user has no recent programs THEN THE SYSTEM SHALL show an empty state in each affected region with a link to Library, or, when the library has no content, a message that the host has not added meditations yet (with a link to Admin only for administrators).
- **FR-029 (US2):** IF one Home region fails to load THEN THE SYSTEM SHALL show an error with retry in that region only and keep all other regions usable.
- **FR-030 (US2):** WHERE the viewport is at least 1024 px wide THE SYSTEM SHALL place the greeting, the pills, and the mood check-in (US8) in a side panel next to the main column, and below the main column on narrower viewports.

#### Library (US3)

- **FR-031 (US3):** THE SYSTEM SHALL show Library programs as content cards (cover or generated artwork, title, description, progress label) grouped by meditation category in category order with "Uncategorized" last, each card opening its Program screen.
- **FR-032 (US3):** THE SYSTEM SHALL offer Library category filter chips consisting of "All" (selected by default), one chip per category that contains at least one program, and "Uncategorized" when uncategorized programs exist, where selecting a chip shows only that group.
- **FR-033 (US3):** WHEN the user submits a Library search THE SYSTEM SHALL show matching meditations as content cards with name, duration, progress label, and download action, and SHALL keep the query in the screen address so reload and back navigation reproduce the results.
- **FR-034 (US3):** WHEN the user opens a Library search result THE SYSTEM SHALL open the Player for that meditation.
- **FR-035 (US3):** WHILE a search is active THE SYSTEM SHALL hide the program groups and offer a clear-search action that restores them.
- **FR-036 (US3):** IF a search has no matches THEN THE SYSTEM SHALL show an empty state that repeats the query and offers the clear-search action.
- **FR-037 (US3):** IF the library has no programs THEN THE SYSTEM SHALL show an empty state stating that the host has not added programs yet, with a link to Admin shown only to administrators.

#### Program (US3)

- **FR-038 (US3):** THE SYSTEM SHALL show the Program screen hero as the screen's single featured element containing the program cover (or generated artwork when none exists), title, description, and overall progress label.
- **FR-039 (US3):** THE SYSTEM SHALL offer one primary Program action that opens the Player on the first meditation in program order that is not completed, labeled "Start" when no meditation has progress, "Continue" when some have progress, and "Play again" (opening the first meditation) when all are completed.
- **FR-040 (US3):** THE SYSTEM SHALL list a program's meditations in program order with name, duration, and progress label, each opening the Player for that meditation.
- **FR-041 (US3):** IF a program has no meditations THEN THE SYSTEM SHALL show an empty state stating the program has no sessions yet and SHALL make the primary Program action unavailable.
- **FR-042 (US3):** WHILE Program data is loading THE SYSTEM SHALL show a loading state instead of a progress label, so no placeholder progress value is displayed.

#### Player (US3)

- **FR-043 (US3):** THE SYSTEM SHALL show the meditation's name as the Player page title, its program's title as a link to that Program screen when the meditation belongs to a program, and artwork using the program cover or generated artwork with the white wash.
- **FR-044 (US3):** THE SYSTEM SHALL provide Player controls for play/pause, skip back 10 seconds, skip forward 10 seconds, seeking with elapsed and total time displayed, playback speed, mark completed, and download, each operable by keyboard and carrying an accessible name.
- **FR-045 (US3):** WHILE the user operates the seek control THE SYSTEM SHALL announce the position to assistive technology as minutes and seconds, not as a raw number.
- **FR-046 (US3):** THE SYSTEM SHALL show Player notes as feed items newest first with their local date and time, and a note input whose add action is unavailable while the input is blank.
- **FR-047 (US3):** WHEN the user chooses to delete a meditation note THE SYSTEM SHALL ask for confirmation in an in-app dialog and keep the note when cancelled.
- **FR-048 (US3):** IF the meditation's audio cannot be loaded or played THEN THE SYSTEM SHALL show a playback error in the Player while keeping notes, mark completed, and download usable.

#### Soundscape (US3)

- **FR-049 (US3):** THE SYSTEM SHALL show Soundscape sounds as tiles grouped by sound category with category filter chips ("All" selected by default), each tile showing the sound's name, duration, an add-or-remove-from-mix action whose label reflects the current state, and a download action.
- **FR-050 (US3):** THE SYSTEM SHALL show the current mix as a panel listing each sound in the mix with its own volume control from 0 to 100 and a remove action, together with a mix control pill holding play/stop and the sleep timer.
- **FR-051 (US3):** WHEN a saved mix is loaded THE SYSTEM SHALL set every sound's volume control to that sound's saved volume.
- **FR-052 (US3):** THE SYSTEM SHALL list saved mixes by name, each with load, rename, and delete actions, where rename uses an in-app rename dialog.
- **FR-053 (US3):** WHEN the user chooses to delete a saved mix THE SYSTEM SHALL ask for confirmation in an in-app dialog naming the mix, and keep the mix when cancelled.
- **FR-054 (US3):** IF no sounds exist or the user has no saved mixes THEN THE SYSTEM SHALL show an empty state in each affected region, stating for sounds that the host has not added any yet (with a link to Admin only for administrators) and for mixes how to save one.

#### Mood (US4)

- **FR-055 (US4):** THE SYSTEM SHALL present the Mood score selector as a single-choice group of five options, each showing its number (1–5) and a short text label, with the selected option distinguished by more than color.
- **FR-056 (US4):** THE SYSTEM SHALL show Mood range chips for Today, Last 7 days, Last 30 days, and All time, where the selected range applies to the insights, the trend chart, and the entry list together.
- **FR-057 (US4):** THE SYSTEM SHALL show Mood insights in a panel with entry count, mean score to one decimal, a 1–5 distribution with count and percentage, and a daily trend chart accompanied by a text alternative listing each day's average.
- **FR-058 (US4):** THE SYSTEM SHALL list mood entries as feed items with score, local date and time, and note, each with edit and delete actions, where delete uses an in-app confirmation dialog.
- **FR-059 (US4):** IF the selected range has no mood entries THEN THE SYSTEM SHALL show a no-data state in place of insights and chart, never a zero-valued chart, with an action that moves focus to the score selector.

#### History (US4)

- **FR-060 (US4):** THE SYSTEM SHALL show the History timeline as feed items, each stating its event type in text (mood entry, meditation note, completed meditation), the meditation title or mood score, the local date and time, and the note when present.
- **FR-061 (US4):** THE SYSTEM SHALL offer History range chips for Today, Last 7 days, Last 30 days, and All time, plus a notes-only toggle whose state is conveyed in text or shape as well as color.
- **FR-062 (US4):** IF the timeline is empty for the chosen range and filter THEN THE SYSTEM SHALL show an empty state that names the active filter and offers links to Library and Mood.

#### Statistics (US4)

- **FR-063 (US4):** THE SYSTEM SHALL show Statistics range chips for Today, Last 30 days, and All time, and four summary figures (total minutes, total practice days, current streak, longest streak), each with a text label and unit.
- **FR-064 (US4):** THE SYSTEM SHALL show each practiced day in the selected range with its date and minutes in text, whether or not a visual representation accompanies it.
- **FR-065 (US4):** IF the selected range has no practice THEN THE SYSTEM SHALL show the four figures with their zero values and a no-practice message with a link to Library.

#### Settings (US5)

- **FR-066 (US5):** THE SYSTEM SHALL present Settings as stacked panels in this order: Profile, Language (US9), Password (US7), Your data, Delete account, with Delete account visually set apart as destructive.
- **FR-067 (US5):** THE SYSTEM SHALL show in the Profile panel an avatar preview with upload and remove actions, where remove asks for confirmation in an in-app dialog, and a display-name field that shows the email as its placeholder hint.
- **FR-068 (US5):** IF a chosen avatar file is not a JPEG, PNG, or WebP image or exceeds 2 MB THEN THE SYSTEM SHALL refuse it before uploading and state the allowed types and the size limit.
- **FR-069 (US5):** THE SYSTEM SHALL show in the Your data panel a JSON download labeled as all data, a CSV download labeled as mood entries only, and a file import whose result report lists every count the import returns.
- **FR-070 (US5):** WHEN account deletion is refused because the user is the last active administrator THE SYSTEM SHALL show the last-active-administrator message in the Delete account panel and keep the user signed in.

#### Admin (US5)

- **FR-071 (US5):** IF a non-administrator opens the Admin address THEN THE SYSTEM SHALL show an admins-only state with a link to Home and SHALL NOT request any administrative data.
- **FR-072 (US5):** THE SYSTEM SHALL divide Admin into three sections (Users, Meditations, Soundscape) selectable with chips, show one section at a time, and keep the selected section across a reload of the screen.
- **FR-073 (US5):** THE SYSTEM SHALL show the Users section as a create-user form and a user list showing avatar, display name, email, role, and status as text (Disabled, Must change password, Active), each with enable-or-disable and delete actions.
- **FR-074 (US5):** WHEN a user is created THE SYSTEM SHALL show the temporary password once in a selectable field with a copy action and the instruction that the user must change it at first sign-in.
- **FR-075 (US5):** WHEN the administrator chooses Disable for a user THE SYSTEM SHALL ask for confirmation in an in-app dialog stating the user will be signed out immediately, while Enable SHALL proceed without confirmation.
- **FR-076 (US5):** WHEN the administrator chooses Delete for a user THE SYSTEM SHALL ask for confirmation in an in-app dialog that names the email and states the deletion permanently erases all of that user's data.
- **FR-077 (US5):** WHEN the administrator renames a sound or changes its category THE SYSTEM SHALL use an in-app dialog in which the category is chosen by name from the existing sound categories.
- **FR-078 (US5):** WHEN the administrator renames a sound category, deletes a sound category, or deletes a sound THE SYSTEM SHALL use an in-app dialog (a rename dialog for the rename, a confirmation dialog for each deletion), where the sound-deletion confirmation states that the sound will be removed from every saved mix.

#### Login and Setup (US5)

- **FR-079 (US5):** THE SYSTEM SHALL present Login, Setup, and the change-password screen (US7) in the unauthenticated layout: the canvas, the brand lockup, and one centered panel with no navigation.
- **FR-080 (US5):** WHEN sign-in is refused for wrong credentials THE SYSTEM SHALL show one generic message and keep the entered email.
- **FR-081 (US5):** WHEN a visitor who is already signed in opens Login THE SYSTEM SHALL open Home instead (or the change-password screen when US7 applies).
- **FR-082 (US5):** WHERE the instance has not been set up yet THE SYSTEM SHALL show on Login a link to Setup.
- **FR-083 (US5):** WHILE the setup status is not yet known THE SYSTEM SHALL show a loading state on Setup and SHALL NOT present an operable setup form.
- **FR-084 (US5):** IF the setup status cannot be determined THEN THE SYSTEM SHALL show an error with a retry action and no setup form.

#### Accessible, responsive, private (US6)

- **FR-085 (US6):** THE SYSTEM SHALL lay out authenticated screens in four tiers per design system section 4.3: at least 1280 px, a labeled sidebar; 1024–1279 px, an icon rail; 768–1023 px, an icon rail with Home's side panel moved below the main column; below 768 px, a compact top bar and a bottom navigation bar.
- **FR-086 (US6):** WHILE navigation is shown as an icon rail THE SYSTEM SHALL expose each item's name to assistive technology and show it on hover and on keyboard focus.
- **FR-087 (US6):** WHILE the viewport is narrower than 768 px THE SYSTEM SHALL show a bottom bar with Home, Library, Soundscape, Mood, and More, where More opens the remaining navigation items (History, Statistics, Settings, Admin for administrators, Sign out) and is marked current when the current screen is one of them.
- **FR-088 (US6):** THE SYSTEM SHALL render every screen from 320 px width (and at 400 % zoom of a 1280 px window) without page-level horizontal scrolling, where any intentionally scrollable row of cards is keyboard reachable and tables reflow into stacked rows.
- **FR-089 (US6):** THE SYSTEM SHALL keep the focused element and the screen's primary action from being covered by the bottom navigation bar or the top bar.
- **FR-090 (US6):** THE SYSTEM SHALL keep text contrast at or above 4.5:1 for body text and 3:1 for large text, and non-text indicators (focus, control boundaries, chart marks) at or above 3:1, measured against the actual composited background including translucent surfaces over the canvas.
- **FR-091 (US6):** THE SYSTEM SHALL show a visible focus indicator on every interactive element, as specified in design system section 8.
- **FR-092 (US6):** THE SYSTEM SHALL convey every selected, current, completed, and disabled state through text, weight, shape, or icon in addition to color.
- **FR-093 (US6):** WHILE the user's operating system or browser asks for reduced motion THE SYSTEM SHALL show no non-essential motion, including hover lifts, scaling, and animated transitions.
- **FR-094 (US6):** WHILE the user's operating system or browser asks for reduced transparency, or the browser cannot render translucent blur, THE SYSTEM SHALL render every surface opaque per design system section 8.
- **FR-095 (US6):** THE SYSTEM SHALL give every interactive element a target of at least 44 × 44 px.
- **FR-096 (US6):** THE SYSTEM SHALL expose navigation, main content, and (on Home) the side panel as landmarks, and offer a skip-to-content link as the first focusable element of every authenticated screen.
- **FR-097 (US6):** THE SYSTEM SHALL give every icon-only control an accessible name and hide decorative artwork and icons from assistive technology.
- **FR-098 (US6):** THE SYSTEM SHALL make every action on every screen, including sliders, chips, dialogs, and seeking, operable with a keyboard alone.
- **FR-099 (US6):** THE SYSTEM SHALL keep all content and functions available when text is enlarged to 200 %.
- **FR-100 (US6):** THE SYSTEM SHALL load every font, icon, artwork, and image from the instance itself or generate it locally, making zero requests to any other origin.
- **FR-101 (US6):** THE SYSTEM SHALL show at most one featured (highest-elevation) element per screen and at most one primary-styled button per panel or dialog.
- **FR-102 (US6, US9):** THE SYSTEM SHALL present every interface string (labels, buttons, messages, dialogs, loading, empty, and error states, page and browser-tab titles, and accessible names) in the active interface language, offering English and Turkish.

#### Password change (US7)

- **FR-103 (US7):** THE SYSTEM SHALL offer in the Settings Password panel a new-password field and a confirmation field with a change action, and on success SHALL announce that the password changed and keep the user signed in.
- **FR-104 (US7):** IF the new password is shorter than 8 characters, or the two entries differ, THEN THE SYSTEM SHALL show the minimum-length rule or the mismatch message respectively and SHALL NOT submit the change.
- **FR-105 (US7):** WHILE the signed-in user is flagged to change their password THE SYSTEM SHALL show the change-password screen in place of every authenticated screen, explaining why the change is required and offering Sign out.
- **FR-106 (US7):** WHEN a user flagged to change their password signs in THE SYSTEM SHALL show the change-password screen before any other screen.
- **FR-107 (US7):** WHEN a required password change succeeds THE SYSTEM SHALL open the remembered screen address or Home and SHALL no longer show the change-password screen for that user.

#### Home mood check-in (US8)

- **FR-108 (US8):** THE SYSTEM SHALL offer on Home a quick mood check-in with the same 1–5 single-choice score selector as Mood (3 preselected) and an optional note limited to 10,000 characters.
- **FR-109 (US8):** WHEN a quick check-in is saved THE SYSTEM SHALL announce the confirmation, reset the check-in, and list the new entry among Home's recent mood entries.
- **FR-110 (US8):** THE SYSTEM SHALL show on Home the user's three most recent mood entries as feed items with score, local date and time, and note, plus a link to Mood, or an empty state inviting a first check-in when the user has no mood entries.
- **FR-111 (US8):** IF saving a quick check-in fails THEN THE SYSTEM SHALL keep the selected score and note and show the reason.

#### Interface language (US9)

- **FR-112 (US9):** THE SYSTEM SHALL offer a language choice between English and Turkish in the Settings Language panel and in the unauthenticated layout (Login, Setup, and the change-password screen), each language named in its own language ("English", "Türkçe").
- **FR-113 (US9):** WHEN the user chooses a language THE SYSTEM SHALL switch all interface text on the current screen to that language immediately, without signing the user out and without discarding unsaved input in other fields on that screen.
- **FR-114 (US9):** WHEN no language has been chosen on this device THE SYSTEM SHALL use the first of English or Turkish that appears in the browser's preferred languages, and English when neither appears.
- **FR-115 (US9):** WHEN a language has been chosen on this device THE SYSTEM SHALL keep using it across reloads, sign-out, and sign-in on that device until another language is chosen.
- **FR-116 (US9):** THE SYSTEM SHALL declare the active interface language on every screen so that browsers and assistive technology read the text in that language.
- **FR-117 (US9):** THE SYSTEM SHALL format dates, times, numbers, percentages, and durations according to the active interface language.
- **FR-118 (US9):** WHEN the server refuses an action with a reason the interface recognizes THE SYSTEM SHALL show that reason in the active interface language, and IF the reason is not recognized THEN THE SYSTEM SHALL show a generic failure message for that action in the active interface language.
- **FR-119 (US9):** IF a piece of interface text has no translation in the active language THEN THE SYSTEM SHALL show the English text instead of an empty space or an internal identifier.

### PRESERVED

- **PB-1:** The Player resumes from the saved position unless the meditation is completed, in which case it starts from the beginning.
- **PB-2:** While playing, the Player reports the position at intervals of at least 5 seconds of media time, immediately on pause, and when the page is hidden or closed.
- **PB-3:** Reaching the end of a meditation records exactly one completion for that play-through, with the start time of the first play (or the end time minus the duration when no play start was observed).
- **PB-4:** The Player offers speeds 0.5×, 0.75×, 1×, 1.25×, 1.5×, 1.75×, and 2× (not remembered across visits) and a mark-completed toggle that sets and clears completion.
- **PB-5:** The operating system's media controls show the meditation title and offer play, pause, skip back and forward 10 seconds, seek to a position, and stop.
- **PB-6:** Soundscape plays each sound of the mix as its own loop with its own volume from 0 to 100; a sound added while the mix plays starts immediately; play is unavailable while the mix is empty; a blocked autoplay shows a message asking the user to press play.
- **PB-7:** The sleep timer is available only while the mix plays, accepts whole minutes from 5 to 120, shows a minutes:seconds countdown announced to assistive technology, fades all sounds to silence over the last 60 seconds and then stops, and cancelling restores the volumes from before the fade.
- **PB-8:** Saving a mix requires a name and at least one sound and stores the current volumes; loading a mix stops playback and does not start it automatically; loading a mix with sounds that no longer exist shows the warning that some sounds are no longer available; rename and delete keep working.
- **PB-9:** Mood keeps scores 1–5 with 3 preselected, notes up to 10,000 characters, create, edit, and cancel; editing keeps the original recorded time; deleting an entry requires confirmation; ranges Today, Last 7 days, Last 30 days, and All time (default All time); insights and trend chart with a text alternative; and loading more entries.
- **PB-10:** History keeps ranges Today, Last 7 days, Last 30 days, and All time (default All time), the notes-only filter, the three event types, and loading more records without duplicates or omissions.
- **PB-11:** Statistics keeps ranges Today, Last 30 days, and All time, the four figures (total minutes, total practice days, current streak, longest streak), and labels each practiced day with the calendar date on which the user practiced in their local time, never shifted by a day.
- **PB-12:** Settings keeps avatar upload (JPEG, PNG, WebP, at most 2 MB) and removal with confirmation, display names up to 64 characters (cleared by saving an empty name), JSON and CSV export, import of JSON and CSV files with a report of mood entries created, mood duplicates skipped, practice records linked, practice records skipped as unmatchable, and practice duplicates skipped, and account deletion that requires typing the exact account email and a confirmation before showing the login screen.
- **PB-13:** Admin stays unusable by non-administrators; the temporary password is shown once and is gone after a reload; every existing action remains: create user, enable, disable, and delete user; upload meditation audio (program required; MP3, WAV, Ogg, Opus; name prefilled from the file name; conversion progress shown); create program with optional cover up to 5 MB; create meditation category; set a program's category; add a meditation to a program; play and download library meditations; add, rename, and delete sound categories (delete refused while the category contains sounds); upload, rename, re-categorize, and delete sounds (deleted sounds leave all mixes); the built-in badge; and sound downloads.
- **PB-14:** Login shows a countdown while throttled and keeps sign-in unavailable until it ends, uses one generic message for unknown email and wrong password, and shows the help text telling the user to ask the host operator to run the reset-password command.
- **PB-15:** Setup never stores the setup secret and clears it after every attempt, requires an administrator password of at least 8 characters, shows the already-set-up message with a link to Login, and after success shows a message with a link to Login without navigating automatically.
- **PB-16:** Downloads remain available wherever they exist today: Library search results, Player, Soundscape sounds, Admin library meditations and sounds, and the Settings data exports.
- **PB-17:** Ownership and privacy behavior of specs 0004, 0006, 0007, and 0008 is unchanged: no screen shows another user's mood entries, notes, timeline, statistics, or exports, and administrators see other users only as the user list (avatar, display name, email, role, status).
- **PB-18:** Existing screen addresses keep working, and a Program or Player address without a valid identifier opens Library.
- **PB-19:** Progress labels keep their meaning: programs show "Completed" or "NN% complete", meditations show "Completed" or "NN%".

## Success Criteria

- **SC-001:** On each of the 10 authenticated screens, the navigation shows exactly one current item (Library on Program and Player), and every navigation destination is reachable from every authenticated screen in at most two activations at each of the four layout tiers.
- **SC-002:** As a member, the Admin entry appears 0 times across all screens; as an administrator, it appears in the navigation on every authenticated screen.
- **SC-003:** Signed out, opening each of the 10 authenticated screen addresses shows the login screen 10 of 10 times; after signing in, the originally requested screen (with its parameters) opens 10 of 10 times; a crafted destination pointing to another site opens Home instead.
- **SC-004:** After confirming Sign out, the login screen is shown and reopening any authenticated address shows the login screen; cancelling Sign out leaves the user on the same screen, still signed in.
- **SC-005:** With not-found, network-failure, and server-error responses simulated for Player, Admin, Library, Program, Mood, History, and Statistics, each case shows an error or not-found state and none shows the login screen.
- **SC-006:** With network responses delayed by 3 seconds, every data region on every screen shows a loading state during the delay; 0 regions are blank at any moment.
- **SC-007:** A brand-new member on an instance with no content visits all screens: every region shows content or an empty state, and every empty state that has a relevant next action offers it.
- **SC-008:** During a walkthrough that triggers every confirmation and rename on every screen, 0 browser-native dialogs appear; deleting a meditation note and deleting a saved mix each require confirmation, and cancelling leaves the item in place.
- **SC-009:** Defect regression set passes 6 of 6: a Library search result opens the Player for that meditation; a loaded mix with volumes 30 and 70 shows 30 and 70; the Player title equals the meditation name; a program with a cover shows that cover on its Program screen; the Setup form cannot be filled or submitted before the status check completes; a sound's category is chosen by name, never by number.
- **SC-010:** Specific refusals are shown verbatim in meaning for 5 of 5 cases: duplicate email on user creation, last active administrator on self-deletion, refusal of an administrator disabling their own account, deleting a sound category that still contains sounds, and an oversized avatar.
- **SC-011:** With the device clock set to 08:00, 14:00, and 20:00, Home shows the morning, afternoon, and evening greeting respectively, with the display name when set and with no name or email when unset; exactly one Home tile is featured.
- **SC-012:** An automated accessibility audit of all 12 screens plus the change-password screen and every dialog reports 0 serious or critical violations, and a keyboard-only tester completes each user story's independent test without a pointer.
- **SC-013:** Contrast measurements on each text and indicator style over the worst-case composited background (bottom-right of the canvas) meet 4.5:1 for body text, 3:1 for large text, and 3:1 for focus indicators and control boundaries, 100 % of samples.
- **SC-014:** At widths of 320, 375, 768, 1024, 1280, and 1440 px, no screen scrolls horizontally at page level, the navigation pattern matches the tier, and every interactive element measures at least 44 × 44 px at 320 px.
- **SC-015:** With reduced motion enabled, no element animates on hover, press, or screen change; with reduced transparency enabled, 100 % of sampled surfaces are opaque.
- **SC-016:** A newly created member signing in with the temporary password sees the change-password screen; opening three other authenticated addresses shows it each time; after a valid change the member reaches Home; afterwards a 7-character password in Settings is refused with the minimum-length rule, and a valid change from Settings lets the member sign in with the new password.
- **SC-017:** A score-only and a score-with-note check-in saved from Home both appear in Home's recent entries and on the Mood screen with identical score, note, and recorded time.
- **SC-018:** Network capture across a full walkthrough of every screen records 0 requests to any origin other than the instance.
- **SC-019:** For each of English and Turkish, every interface string on every screen and dialog appears in that language: 0 strings in the other language and 0 internal identifiers, excluding user- or host-entered content, built-in sound names, and proper names.
- **SC-020:** Each PB-1 to PB-19 passes its regression check on the redesigned screens, 19 of 19.
- **SC-021:** For each of the 12 screens, a reviewer finds every pattern listed for that screen in design system section 11 present, apart from the deliberate exceptions recorded under Assumptions.
- **SC-022:** Switching the language in Settings and on Login changes the current screen's text immediately with the user still signed in; the choice survives a reload and a sign-out/sign-in on the same device 3 of 3 times; first visits with the browser preferring Turkish, English, and German show Turkish, English, and English respectively.
- **SC-023:** With the interface in Turkish, dates, times, numbers, and durations on Home, Mood, History, and Statistics use Turkish formatting, and the five specific refusals of SC-010 appear in Turkish.

## Edge Cases

- **Session expires mid-task:** any refused request redirects to Login and remembers the screen (FR-009); unsaved form input on that screen is lost, and nothing is saved partially.
- **Already signed out when choosing Sign out:** the login screen is shown without an error (FR-008).
- **Remembered destination is another site, a malformed address, or Login itself:** Home opens after sign-in (FR-010).
- **Invalid identifier in a Program or Player address** (missing, non-numeric, zero): Library opens (PB-18); a valid-looking identifier with no item shows the not-found state (FR-012).
- **Network lost after a screen loaded:** the failing action shows an error with retry, while already-loaded content stays visible.
- **Avatar image fails to load:** the account button falls back to the initial, then to the generic icon (FR-006).
- **Display name of 64 characters, or a long program or meditation name:** text wraps or truncates without overlapping other elements or causing horizontal scrolling, and the full text stays available to assistive technology.
- **Browser storage unavailable (for example strict private browsing):** the language follows the browser preference on each visit and no error is shown (FR-114).
- **Language switched while a dialog is open:** the dialog's text switches too and the dialog stays open (FR-113).
- **Longer Turkish strings:** labels, chips, buttons, and navigation wrap or truncate without overlap or horizontal scrolling in both languages, with the full text available to assistive technology.
- **Turkish dotted and dotless i in names:** the initial keeps Turkish casing and the greeting shows the name exactly as entered (FR-006).
- **Device clock exactly at 05:00, 12:00, or 18:00:** the later greeting applies (FR-022).
- **Many items** (hundreds of programs, sounds, users, or timeline records): screens remain usable, the timeline keeps paging, and nothing is silently omitted.
- **Program whose meditations are all completed:** the primary action reads "Play again" and opens the first meditation (FR-039).
- **Library category with no programs:** it gets no chip and no empty group heading (FR-032).
- **Search submitted with only whitespace:** treated as clearing the search.
- **Mix sound removed by an administrator while the user has the mix loaded:** the next load shows the unavailable-sounds warning (PB-8); the remaining sounds play.
- **Mix deleted in another tab:** the next action on it shows an error and the saved-mix list refreshes.
- **Navigating away from Player or Soundscape while playing:** playback stops; the Player's position is reported on page hide (PB-2); continuing playback across screens is out of scope.
- **Browser without operating-system media controls:** the Player works without them.
- **Browser that cannot render translucent blur:** opaque surfaces (FR-094).
- **On-screen keyboard on a phone:** the focused field and the primary action stay visible above the bottom bar (FR-089).
- **Dialog open while the session expires:** the dialog closes and the login screen is shown.
- **Double activation of a destructive confirm:** the action runs once (FR-020).
- **Last active administrator disables or deletes themselves from Admin:** the specific message is shown and the account stays intact (FR-015).
- **Administrator tries to disable their own account:** the server always refuses self-disabling, even when other administrators exist; the specific refusal is shown and the account stays active (FR-015).
- **Temporary password shown, then the admin reloads:** it is gone (PB-13).
- **Import of a malformed file:** the specific rejection message is shown and the report is not shown.
- **Quick check-in note over 10,000 characters:** the limit is stated, saving stays unavailable until the note fits, and the text is kept (FR-108).
- **Flagged user on Login, Setup, or the change-password screen itself:** no redirect loop; the change-password screen is the only destination until the change succeeds (FR-105).
- **Setup opened on an already set-up instance by a signed-in user:** the already-set-up message with a link to Login is shown (PB-15).
- **Practice crossing local midnight, or a timezone or daylight-saving change between visits:** dates follow the current local timezone and the date reported for each practiced day (PB-11).

## Assumptions

- This is purely a web-interface change: no API endpoint, data, or schema is added or changed. Every requirement is satisfied by existing endpoints, including the existing but unused sign-out and own-password-change endpoints.
- Global navigation follows design system section 11 exactly; Program and Player are reached from content and mark Library as current.
- On screens narrower than 768 px the bottom bar holds Home, Library, Soundscape, Mood, and More (design system 4.3 allows 4–5 items); the remaining items live behind More.
- The side panel (aside) appears only on Home; other screens use the main column only. This settles design system open decision 3 for this change.
- The header search field appears on Library only, because search is a library capability; other screens show no search. There is no notification button, because the app has no notifications.
- Sign-out asks for confirmation, as design system 5.2 calls it "a confirmed action".
- After a redirect to Login, the requested screen is remembered and reopened after sign-in, restricted to this application's own addresses so it cannot become an open redirect.
- Login links to Setup only when the setup status reports no administrator; signed-in visitors opening Login go to Home.
- The Home greeting uses the display name when one exists and otherwise no name; the email is never used, to avoid exposing it on a glanceable screen.
- Home tiles are Library (featured), Soundscape, Mood, History, and Statistics; Settings and Admin stay in navigation only (design system 4.2 caps tiles at 4–5).
- The continue-program pill's play action opens the next meditation directly (the data already identifies it); its title opens the program.
- Home also shows the three most recent mood entries as feed items, as design system section 11 describes recent mood notes on Home; this reads the user's own data only.
- Home's "See all" for recent programs keeps going to History, as today.
- Home's recent-programs row has no category chips, unlike design system section 11 suggests: filtering at most five cards adds nothing. This is a deliberate exception for SC-021.
- Library hides program groups during a search and restores them with clear-search (today they stay visible beneath the results); categories without programs get no chip or heading.
- The Program primary action opens the first not-completed meditation in program order.
- Admin is split into Users, Meditations, and Soundscape sections; the selected section survives a reload, and the plain Admin address opens Users.
- Disabling a user requires confirmation because it signs that user out immediately; enabling does not, because it is harmless and reversible.
- Sound categories in the admin rename/re-categorize dialog are chosen by name, replacing today's prompt for a raw category number.
- "At most one primary button per view" (design system 5.13) is read as per panel or dialog, because Settings and Admin legitimately hold several independent forms.
- Statistics day labels are already correct today: the server reports local calendar dates, and the interface shows them unchanged. The redesign must keep this, so it is a preserved behavior (PB-11), not a defect fix.
- The change-password form asks for the new password twice but not the current one, because the existing endpoint does not check the current password. Requiring it would need an API change, which is out of scope.
- Forced password change is enforced by the interface only; the server does not block other requests for flagged users. This closes the user-visible gap in spec 0004 SC-001 and spec 0001 US4 but is not a server-side security boundary.
- Copy quoted in this spec (for example "Good morning", "Start", "Delete mix") is the English reference text; every quoted string has a Turkish counterpart.
- The language preference is stored per device and browser, not on the user account, because a per-account preference would need an API and schema change, which this spec excludes. Two devices of the same user can therefore show different languages.
- Host- and user-entered content (program, meditation, category, sound, and mix names, notes, display names) and the names of the built-in sound set are content, not interface text, and are not translated.
- This change replaces the assumption in spec 0004 that the interface language stays Turkish: the interface is now bilingual with a user choice.
- English is the reference language: when a translation is missing, the English text is shown (FR-119).
- Exact token values, the typeface, and the icon set are implementation choices governed by design system sections 3, 6, and 12; this spec requires only the observable outcomes.

## Out of Scope

- Dark mode or any theme switch.
- The reference design's AI assistant or chat, community features, premium tiers, bookmarks or saved items, and notifications.
- A persistent mini-player that keeps playback running across screen navigation.
- New administrative capabilities such as changing roles or deleting meditations and programs.
- Editing meditation notes, renaming or deleting meditation categories, and adding a Last 7 days range to Statistics.
- Links from History events to the Player or Program.
- Languages other than English and Turkish, and a language preference that roams with the user account across devices.
- Translating host- or user-entered content and the built-in sound names.
- Any new or changed API endpoint, data field, or schema, including a current-password check on password change and server-side enforcement of the must-change-password flag.
- Offline support, installable-app behavior, and background playback changes.

## Clarifications

### Session 2026-10-04

- Q: Which single language should all interface text use — English, Turkish, or keep today's mix? → A: The interface language must be fully switchable; ship English and Turkish first (US9, FR-102, FR-112–FR-119).
- Q: Are self-service password change and the forced change of a temporary password part of this redesign, or deferred to a separate spec? → A: Included (US7, FR-103–FR-107, SC-016).

## Deviations

### 2026-10-04 — Timeline pagination requires a backend repair

- **Requirement affected:** FR-060–062, PB-10, and the History load-more acceptance criterion.
- **Observed blocker:** The existing `GET /api/timeline` cursor cannot be parsed after the first page because its RFC 3339 timestamp contains colons, while the server splits the cursor at the first colon. Its continuation comparison also follows the wrong direction for the endpoint's descending sort order. A real second page therefore returns HTTP 400.
- **Impact:** The approved UI-only implementation cannot satisfy History pagination against the real API. The History UI preserves the received cursor and de-duplicates appended records, but cannot repair an invalid backend cursor.
- **Proposed resolution:** Make the smallest compatible backend bug fix: parse the unchanged `RFC3339:TimelineKind:id` cursor from the right and retain only records strictly after the cursor in the actual descending order. A scoped implementation and owning-router regression test exist in an isolated local worktree but have not been applied to this working tree.
- **Status:** **resolved 2026-10-04.** The owner approved the minimal backend repair (cursor parsed from the right; continuation follows the endpoint's descending order). Applied in `src/modules/mood-tracker/presentation/src/router.rs` with a router regression test in `router.tests.rs`. History load-more verified against the real server: an end-to-end run (32 passed).
