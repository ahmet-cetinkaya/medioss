---
status: approved
spec: specs/0007-profile-settings/SPEC.md
capability: profile-settings
---

# Plan — Profile Settings (spec 0007)

## Goal

Ship the profile surface defined by `specs/0007-profile-settings/SPEC.md`: display name (set/change/clear) and upload-only avatars (jpg/png/webp ≤ 2 MB, server-normalized square ≤ 256 px, stored under `media/{user_id}/`), visible to the owner and the admin list. No new page: the existing `/settings` (Turkish UI) gains a profile card; the header shows avatar + display name.

## Context

Paths and conventions discovered (main @ `c60a537`):

- Identity layering: `domain` (`User` in `src/modules/identity/domain/src/user/user.rs`, private fields + builder-style constructors + `with_disabled` style mutators) → `application` (one file per command in `src/modules/identity/application/src/commands/*.rs`, each with a sibling `.tests.rs`; ports in `src/modules/identity/application/src/ports/user_repository.rs`) → `presentation` (`src/modules/identity/presentation/src/router.rs` + `users_lifecycle.rs` handlers; `CurrentActor`, `pub struct AdminActor(pub Actor)` extractors; `ensure_browser_origin` on every mutation; route tests in `router.tests.rs` with `seeded_app`, `oneshot`, `Origin` header) → `infrastructure` (`src/modules/identity/infrastructure/src/persistence/users/` SeaORM entity + `repository.rs`; SQL DDL migration `migrations/m20260908_000001_identity_create_schema.rs` — amended in place per the one-migration invariant, app unreleased).
- Multipart precedent: `src/modules/meditation/presentation/src/handlers_audio.rs` — cap enforced twice (declared `content_length` + running byte counter), `read_field_capped` helper; `DefaultBodyLimit` is NOT globally set; axum `Multipart` extractor with the `multipart` feature already enabled in meditation's presentation crate.
- File serving: web-api composes static files via `ServeDir` (`src/apps/web-api/src/router.rs`); per-request guarded serving has no precedent — avatar `GET` is a new authenticated route, not a static dir.
- `media/{user_id}/` root already exists (`src/apps/web-api/src/user_deletion.rs` wipes it; meditation download cache lives there). Avatars reuse it; deletion wipes them via FR-027 with zero new code.
- Frontend: Svelte 5 + Astro pages (`src/apps/web-ui/src/pages/*.astro` → `src/lib/components/*.svelte`); `settings.svelte` exists (danger zone only); typed API clients in `src/lib/api/identity.ts` (fetchUsers/createUser/setUserStatus/deleteUser/deleteOwnAccount) and `meditation.ts::fetchMe` (id/email/role — note: `me` response currently lacks `must_change_password` etc. in the UI type, backend returns more).
- Workspace deps (`Cargo.toml`): no `image` crate yet — must be added to `[workspace.dependencies]` (pinned, default-features-off style consistent with the rest).
- Session/actor: `kernel::Actor` (`Actor::new(UserId, Role)`); identity deps are generic `<R, H, C, A>`; the avatar file write needs `media_root` — a new dependency the identity presentation layer must not own: it takes an `AvatarStore` port instead.

## Decisions

1. **Numbering/authoritative doc** — spec 0007 is the contract; this plan implements FR-030..FR-036 / SC-001..SC-007 verbatim. Constitution check: repo has no other instruction files beyond CLAUDE.md safety blocks and the spec folder convention (`specs/<NNNN>-<slug>/`); living-spec policy (`status: living`) means amendments land in the spec's Clarifications/Deviations sections when implementation reveals drift — not silently.
2. **Domain model** — `User` gains `display_name: Option<DisplayName>` and `avatar: Option<AvatarRef>` (new value objects in `domain/src/user/display_name.rs` and `avatar_ref.rs`; definitions live in their own files per the mod.rs policy). `DisplayName`: trimmed, 1..=64 chars (UTF-8 scalars, not bytes), not unique. `AvatarRef`: stores the normalized extension only (`"webp" | "png"`); path is always derived `media/{user_id}/avatar.{ext}`.
3. **Application layer** — three new commands, each side-effect-checked and unit-tested like their siblings: `set_display_name(repository, actor, name)` (trim, validate, clear-on-empty), `replace_avatar(repository, actor, ext)` (persists `avatar_ext`, returns previous ext so the caller can unlink), `clear_avatar(repository, actor)` (returns previous ext). A new port `AvatarStore` (application) abstracts file write/delete/normalize: `normalize(bytes) -> Result<(Vec<u8>, &'static str), AvatarError>` (decode via `image`, square center-crop, resize ≤ 256 px, encode webp — fallback png when webp encode fails on this image), `write(user_id, ext, bytes)`, `remove(user_id, ext)`. Infrastructure provides `FileAvatarStore { media_root }`; presentation receives it as a generic `V: AvatarStore` in `IdentityDependencies` — keeps web-api as the only crate touching disk.
4. **Migration** — amend `m20260908_000001_identity_create_schema.rs` in place: `users` gains `display_name TEXT NULL` and `avatar_ext TEXT NULL CHECK (avatar_ext IN ('webp','png'))`. Dev DBs re-initialize (spec assumption; same call as the `disabled` column).
5. **API surface** (paths from the spec; `PUT` for upload — idempotent replace semantics):
   - `PATCH /identity/me/profile` `{display_name: string}` → `200` + updated profile `{id, email, role, display_name, has_avatar}`; `400` invalid shape.
   - `PUT /identity/me/avatar` multipart field `file` → `201`; `400` undecodable/wrong type/SVG/text; `413` when declared or counted size > 2 MB (rejected before decode); `401` anonymous. Double size guard copied from handlers_audio.
   - `DELETE /identity/me/avatar` → `204`, idempotent.
   - `GET /identity/avatar/{user_id}` → owner or admin only (`403` other member, `404` no avatar, `401` anonymous); served with `Cache-Control: private, max-age=86400` and the stored content type from `AvatarStore::read(user_id, ext)` — read joins the port so presentation never touches paths.
   - `GET /identity/me` and `GET /identity/users` items gain `display_name` + `has_avatar`.
   - All new mutations keep `ensure_browser_origin` + `CurrentActor`/`AdminActor` conventions; the avatar GET is a read (no origin check, like other GETs).
6. **Magic bytes** — decode is the validator: `image::load_from_memory` accepts jpg/png/webp by sniffing, rejects everything else; explicit reject when `image::guess_format` says SVG (the crate does not decode SVG — belt-and-braces error mapping so it never returns a confusing generic 400).
7. **Frontend** — `settings.svelte` gains a profile card (display name input + save, avatar upload `<input type="file" accept="image/jpeg,image/png,image/webp">` with client-side 2 MB pre-check and Turkish errors, avatar preview + "Kaldır" button with confirm). Header (the shared top bar in `settings`/home components — same fetch pattern as `fetchMe`) shows avatar (with cache-busted URL `/api/identity/avatar/{id}?v={hash}`) or initial-letter placeholder derived from display name ?? email, plus display name. Admin page table gains two columns (display name, avatar thumbnail via the same guarded endpoint — admin session passes FR-033). `identity.ts` gains typed clients; `fetchMe` return type extended.
8. **Deletion coverage (SC-005)** — no new deletion code: extend the existing delete integration/route tests' wipe assertions to check `avatar.*` absence under `media/{user_id}/` after delete.
9. **Testing strategy** — command unit tests (trim/limit/clear, avatar replace-returns-previous-ext); route tests for every SC row (reuse `seeded_app` — extend `IdentityDependencies` generics in tests with a `NullAvatarStore` fake + counting assertions); infrastructure repository tests for the two new columns; UI: build only (no UI unit framework exists).
10. **No user-owned blockers** — all three product decisions (upload-only, visibility, 2 MB) were answered 2026-10-02 and recorded in the spec's Clarifications.

## Scope

**In:** domain value objects + `User` fields; migration amendment; three commands + `AvatarStore` port + `FileAvatarStore`; five route changes (2 new resources + 2 DTO extensions); `image` workspace dep; settings page profile card; header avatar/name; admin table columns; tests at every layer; spec Deviations/Clarifications updates if reality forces shape changes.

**Out:** preset avatar sets; gravatar/external sources; quota accounting; member-to-member visibility surfaces; email/theme/notification settings; UI unit tests; pagination on the admin list.

## Tasks

1. **Domain: DisplayName + AvatarRef + User fields.** Add `display_name.rs`, `avatar_ref.rs`; extend `User` (constructor defaults `None`, `with_display_name`, `with_avatar_ext` mutators, getters). AC: `cargo test -p identity-domain` green with new tests covering 1/64-char bounds, unicode trim, empty→None, invalid ext rejected.
2. **Migration + repository.** Amend DDL in place; entity + SeaORM mapping for both columns; `UserRepository` gains `set_profile(id, display_name, avatar_ext)` (single UPDATE covering both profile commands' persistence). AC: infrastructure repository tests persist and reload both fields round-trip; migration test still passes.
3. **Application: set_display_name command.** Validation + persistence via `set_profile`. AC: command tests — set/trim/unicode/over-length `400`/whitespace-only `400`/empty clears; anonymous-actor shape not needed (route layer owns 401).
4. **Application: avatar commands + AvatarStore port.** `replace_avatar`, `clear_avatar`, `AvatarStore` trait (normalize/write/read/remove) in its own file. AC: command tests with a fake store assert previous-ext handoff (replace unlinks old after persist) and clear idempotence.
5. **Presentation: routes + DTOs.** `PATCH /me/profile`, `PUT/DELETE /me/avatar`, `GET /avatar/{id}`; extend `me` + `users` list DTOs; `IdentityDependencies` gains `avatar_store: V` and `media` origin unchanged. AC: `cargo check -p identity-presentation`; route handlers keep origin guard + actor extractors.
6. **Presentation: route tests (SC-001..SC-004, SC-006).** Every row of the spec's SC table as an oneshot test; counting fake store for the 2 MB/corrupt/SVG/double-upload-replaces cases (exactly-one-file assertion). AC: `cargo test -p identity-presentation` green; spec SC checklist maps 1:1 to test names.
7. **web-api: FileAvatarStore + wiring.** `image` crate to workspace + web-api deps (webp+png+jpeg features, no default extras); `FileAvatarStore` under `src/apps/web-api/src/avatar_store.rs` (module, not mod.rs definitions — follows the same rule); normalize pipeline (center-crop square, ≤ 256 px, webp → png fallback) with unit tests over synthetic images; wire into `identity_dependencies`. AC: unit tests prove decode-reject (text bytes, SVG) and size guard paths; workspace check green.
8. **Deletion wipe assertion (SC-005).** Extend the existing user-deletion tests: seed an avatar file, delete the user, assert `media/{user_id}/` empty including `avatar.*`. AC: test green without any new product code.
9. **UI: settings profile card.** `identity.ts` clients (`updateProfile`, `uploadAvatar`, `removeAvatar`, extended `fetchMe`); `settings.svelte` card with Turkish copy, client-side checks, error mapping (400/413/401). AC: `bun run build` green; manual QA on dev server (upload → preview; remove → placeholder).
10. **UI: header avatar/name + admin columns.** Header component(s) render avatar-or-initials + display name; admin table gains the two columns using the guarded avatar endpoint. AC: build green; admin list shows both for a seeded profile (manual QA).
11. **Gates + live QA + spec sync.** fmt, clippy `-D warnings`, `cargo test --workspace`, boundary check, UI build; live QA script covering SC-007 (upload from browser flow simulated via multipart curl: 201 → header data present; remove → 204; display name change reflected in `me` + admin list). Spec Deviations updated if any endpoint shape moved. AC: all gates green; PR ready.

Dependency order: 1 → 2 → (3, 4) → 5 → 6 → 7 → 8; 9, 10 after 5; 11 last.

## QA / Evidence

- Per-layer test runs quoted in the PR body; route-test names mapped to SC ids.
- Live QA transcript: curl multipart upload (valid 100 KB png, 3 MB png, text-file-as-png, SVG) + profile PATCH round-trip + admin list render check + delete-user-wipes-avatar file-system assertion.
- Gate logs: fmt/clippy/test/boundary/ui (42-suite baseline + new suites).

## Traceability

| Spec item | Task(s) | Evidence |
| --- | --- | --- |
| FR-030 (profile PATCH) | 1, 3, 5 | T1 domain tests; T3 command tests; T6 SC-001 route tests |
| FR-031 (avatar PUT) | 4, 5, 7 | T4 command tests; T6 SC-002 route tests; T7 decode/normalize tests |
| FR-032 (avatar DELETE) | 4, 5 | T4 clear-idempotence; T6 SC-003 delete row |
| FR-033 (avatar GET guarded) | 5, 7 | T6 SC-003 200/403/404/401 rows |
| FR-034 (me DTO) | 5, 6 | T6 SC-004 me row |
| FR-035 (users list DTO) | 5, 6, 10 | T6 SC-004 users row; T10 admin columns |
| FR-036 (deletion wipes avatar) | 8 | T8 wipe assertion |
| SC-001..SC-004, SC-006 | 6 | one test row each in router.tests.rs |
| SC-005 | 8 | extended deletion test |
| SC-007 (gates + live QA) | 9, 10, 11 | gate logs; live QA transcript |

No task lacks a criterion; no criterion lacks a task.

## Final Verification

`bash` gates script: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && scripts/check-boundaries.sh && (cd src/apps/web-ui && bun run build)` — then the live QA script; both quoted in the PR. Every SC id in the spec maps to at least one green test or recorded manual step before the PR is marked ready.

## Blockers

None — user-owned decisions (upload-only, owner+admin visibility, 2 MB) resolved 2026-10-02 and recorded in the spec.
