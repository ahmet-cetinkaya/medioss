---
status: complete
spec: specs/0004-user-management/SPEC.md
---

# Implementation Plan: Kullanıcı Yönetimi (Admin UI + Liste)

## Goal

Onaylı spec 0004'ü teslim et: admin'in web arayüzünden kullanıcı ekleyebilmesi ve hesap listesini görebilmesi. `POST /api/identity/users` (FR-013) zaten mevcut ve testli; bu plan yalnızca (1) `GET /api/identity/users` admin-only liste uç noktasını, (2) admin sayfasında kullanıcı yönetimi bölümünü (form + liste + tek kullanımlık şifre gösterimi) ekler. Hiçbir mevcut identity davranışı değişmez (PRESERVED: FR-004/012/013/014/018).

## Context

- **Mevcut API:** `POST /identity/users` router.rs:55, `create_user` komutu → `insert_member` → `User::new_temporary` (member rolü, must-change-password, 24h geçici hash). 403/409/400 yolları komut + rota testleriyle güvence altında.
- **UserRepository** portunda `list` yok — `count`, `find_by_email/id`, `insert_initial_admin_if_uninitialized`, `insert_member`, `apply_credential_change` var. Yeni `list_users` query'si port genişletmesini gerektirir.
- **UI:** Astro pages (`pages/admin.astro` → `components/admin.svelte`), rol kontrolü `fetchMe()` ile satır 63'te (`me.role !== "admin"` → içerik yok). `lib/api/` altında identity istemcisi yok; `fetchMe`/`login` `api/meditation.ts` içinde duruyor (teknik borç, bu planlık kapsam değil ama yeni istemci `lib/api/identity.ts`'e yazılır).
- **Test düzeni:** presentation'da `router.tests.rs` (fake repository ile rota testleri), infrastructure'da `repository.tests.rs` (in-memory SQLite), application'da query testleri.
- Konvansiyon: plan spec klasöründe `PLAN.md`; commitler İngilizce conventional, gövde FR referansları taşır.

## Tasks

### T1 — Application: `list_users` query

`identity/application/src/queries/list_users.rs` (+ test dosyası, `mod.rs` kaydı): `list_users(repository) -> Result<Vec<User>, ApplicationError>`; boş repo → `vec![]`. `UserRepository` portuna `async fn list(&self) -> Result<Vec<User>, RepositoryError>` eklenir; `fake_repository.rs` ve `infrastructure/persistence/repository.rs` (SQL: `SELECT ... ORDER BY email ASC`) uygular. Domain değişikliği yok (User zaten tüm alanları taşıyor).

Kabul: `cargo test -p identity-application` yeşil; `queries/list_users.tests.rs` (boş → vec![], seed'li → email asc sıralı döner).

### T2 — Presentation: `GET /identity/users` (admin-only)

`identity/presentation/src/router.rs`: yeni rota `GET /identity/users` → `AdminActor` extractor'ü (mevcut `CurrentActor` + rol kontrolü pattern'i — soundscape router'daki `AdminActor` ile aynı şekil) + `list_users` çağrısı → `200` `[{id, email, role, must_change_password}]` (şifre hash'i asla serileştirilmez; `created_at` zaten mevcut). Origin kontrolü: GET için gerekmez (mevcut GET rotaları origin kontrolü uygulamıyor — login/me ile tutarlı).

Kabul: `router.tests.rs`'a 3 test: admin → 200 + liste doğru sıralı; member → 403; anonim → 401. Mevcut `POST /identity/users` testleri etkilenmez.

### T3 — Web UI: identity istemcisi + admin sayfası kullanıcı bölümü

`lib/api/identity.ts` (yeni): `fetchUsers()`, `createUser(email, password)` — `request()` helper'ı `api/meditation.ts`'ten import/re-export edilir (helper'ın kendisi kopyalanmaz). `admin.svelte`: mevcut düzenin devamına "Kullanıcılar" kartı — form (e-posta + geçici şifre girdileri), 409/400 mesajları form altında aynı dilde (FR-023), başarılı oluşturmada geçici şifre tek seferlik görüntülenir (FR-025a; kopyalanabilir, sayfa yenilendiğinde kaybolur), liste (id, e-posta, rol, must-change göstergesi) `fetchUsers()` ile, üye rolü zaten satır 63'te engelleniyor (FR-024 karşılanmış durumda — sadece doğrulama kalır).

Kabul: `bun run build` temiz; elle QA (T5'te).

### T4 — Gates + commit

`cargo fmt --all`; `nix-shell -p rustfmt clippy --run "cargo clippy --workspace --all-targets -- -D warnings"`; `cargo test --workspace`; `cd src/apps/web-ui && bun run build`. Tek commit (tek cohesion'lu değişiklik): `feat(identity): admin user management UI and user listing` — gövde FR-022..FR-025a + PRESERVED referansları. Commit yalnız kullanıcı isterse (repo kuralı).

### T5 — Canlı QA (SC-001..SC-004a)

`/tmp` altında taze data dir + sunucu: admin kurulumu → admin formundan kullanıcı oluştur (SC-001: 201, listede) → yeni kullanıcı geçici şifreyle login → `must_change_password: true` (SC-004a) → şifre değiştir → normal kullanım. Aynı e-posta tekrar (farklı case) → formda 409 mesajı (SC-002). Üye oturumuyla admin sayfası → liste çekilmez (SC-003). Rota testleri zaten SC-004'ü kapsıyor.

## DoD

- Tüm gates yeşil (fmt, clippy `-D warnings`, `cargo test --workspace`, UI build).
- Canlı QA senaryoları SC-001..SC-004a geçer.
- Traceability tablosundaki her FR/SC için kanıt mevcut.

## Traceability

| ID | Task(s) | Kanıt |
| --- | --- | --- |
| FR-022 | T3 | QA: form → 201 + listede; rota testi T2'de değil ama create rotası mevcut |
| FR-023 | T3 | QA: 409/400 formda gösterilir, form korunur |
| FR-024 | T4 | Mevcut satır 63 gate + QA üye oturumu |
| FR-025 | T1, T2 | Rota testleri (200/403/401) + QA |
| FR-025a | T3 | QA: şifre tek seferlik görünür |
| SC-001..005 | T4, T5 | Gates + canlı QA çıktıları |

## Risks / Rollback

- Port genişletmesi (`UserRepository::list`) breaking'dir ama yalnızca iki impl (fake + SQLite) var; ikisi de bu plan içinde güncellenir. Risk düşük.
- `created_at` repository SELECT/serialize yolunda zaten mevcut (repository.rs:71); şema değişikliği yok — migration gerekmez.
- Geri alma: tek commit revert. API geriye dönük uyumlu (yalnız eklemeler).
- Kimlik doğrulama/throttle/argon2 yollarına dokunulmuyor — PRESERVED listesi.
