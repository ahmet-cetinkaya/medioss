---
status: living
capability: user-management
plan: specs/0004-user-management/PLAN.md
supersedes:
---

# M4 — Kullanıcı Yönetimi

## Intent

Yönetici (admin) kimliği doğrulanmış kullanıcıları uygulamanın hiçbir yerine dokunmadan bir
ekran üzerinden ekleyebilsin ve hesap listesini görebilsin. API yüzeyi (`POST
/api/identity/users`) M1'den beri mevcuttur; bu spec, admin web UI yüzeyini ve liste uç
noktasını sözleşme altına alır. Tek instance'lı uygulamada kullanıcı ömrü yönetici ile
başlar: self-servis kayıt yok (PRD), e-posta altyapısı yok (PRD) — geçici şifre yöneticinin
ekranında bir kez gösterilir ve kişisel kanaldan iletilir.

## User Stories

### US1 — Yönetici, kullanıcı ekler (Priority: P1)

**Bireysel test:** Yönetici olarak admin sayfasını aç; e-posta ve geçici şifre girip gönder; yeni kullanıcı listede görünür ve oluşturulan hesapla ilk girişte şifre değişikliği istenir.

- **Given** yönetici oturumu ve kullanılmayan bir e-posta, **when** kullanıcı oluşturulursa, **then** istek `201` döner, hesap `member` rolüyle ve must-change-password işaretiyle oluşur, ve UI tek kullanımlık şifreyi bir kez, yalnızca bu ekranda gösterir.
- **Given** zaten mevcut bir e-posta (büyük/küçük harf farkı dahil), **when** kullanıcı oluşturulursa, **then** UI `409` hatasını aynı dilde bir hata mesajı olarak gösterir ve form temizlenmez.
- **Given** 8 karakterden kısa bir geçici şifre, **when** form gönderilirse, **then** API `400` döner ve UI doğrulama hatasını alanın altında gösterir.
- **Given** üye oturumu, **when** admin sayfası açılmaya çalışılırsa, **then** sayfa yönetici olmayanlara içeriği göstermez ve yönetim aksiyonları 403 ile reddedilir.

### US2 — Yönetici, kullanıcıları listeler (Priority: P2)

**Bireysel test:** Yönetici olarak admin sayfasını aç; hesap listesi e-posta, rol ve durum ile görüntülenir.

- **Given** yönetici oturumu, **when** admin sayfası açılır, **then** mevcut hesaplar (id, e-posta, rol, must-change-password durumu, oluşturma zamanı) e-posta alfabetik sırada listelenir.
- **Given** üye veya anonim istemci, **when** liste uç noktası çağrılırsa, **then** istek sırasıyla 403/401 ile reddedilir.

## Requirements

### ADDED

- FR-001: WHEN yönetici oturumlu bir istemci admin sayfasındaki formu kullanılmayan bir e-posta ve en az 8 karakterlik bir geçici şifreyle gönderirse THE SYSTEM SHALL `201` ile hesabı oluşturup UI'da listeye eklesin; hesap member rolünde ve must-change-password işaretli doğar.
- FR-002: WHEN oluşturma isteği `409` (e-posta çakışması) veya `400` (doğrulama) döndürürse THE SYSTEM SHALL formdaki girişleri koruyup mesajı formun yanında aynı dilde göstersin.
- FR-003: WHEN üye rolündeki bir kullanıcı admin sayfasına erişmeye çalışırsa THE SYSTEM SHALL sayfayı boş/erişilemez gösterip API çağrılarını yapmadan reddetsin (API zaten 403 döner; UI kaba dokunmadan gizler).
- FR-004: WHEN admin sayfası yüklendiğinde THE SYSTEM SHALL mevcut kullanıcıları (id, e-posta, rol, must_change_password, oluşturma zamanı) e-posta alfabetik sırada çekebilsin; liste uç noktası yalnızca yönetici erişimine açıktır.
- FR-005: WHEN geçici şifre gösterim alanı kullanılırsa THE SYSTEM SHALL şifreyi yalnızca oluşturma anında bir kez görüntüleyip sonraki render'larda ve sayfa yenilemelerinde erişilemez kılsın; şifre yöneticinin ekranında kalıcı olarak saklanmaz.

## Success Criteria

- SC-001: Yönetici, admin sayfasından bir kullanıcı oluşturur; istek `201`, hesap listede, yeni kullanıcı ilk girişte şifre değişikliğine zorlanır (canlı QA: admin ekler → üye geçici şifreyle girer → şifre değişimi istenir).
- SC-002: Aynı e-posta tekrar (farklı case) gönderildiğinde UI çakışma hatası gösterir, form korunur; API seviyesinde `409` doğrulanır.
- SC-003: Üye oturumuyla admin sayfası açıldığında kullanıcı listesi çekilmez ve oluşturma aksiyonu `403` alır; rota testleriyle kanıtlanır.
- SC-004: Rota seviyesi testler: admin `POST /api/identity/users` → 201; üye → 403; anonim → 401; `GET /api/identity/users` → admin 200 (sıralı) / üye 403.
- SC-005: Tüm kapılar (fmt, clippy `-D warnings`, `cargo test --workspace`, UI build) yeşil; canlı doğrulama tamamlanır.

## Edge Cases

- Geçici şifre, yöneticiye yalnızca ekranda gösterilir; sayfa yenilendiğinde tekrar görünmez.
- E-posta büyük/küçük harf farklarıyla girilirse, API normalizasyonuyla `409` tetiklenir; UI mesajı hem 409 hem 400 için aynı formda gösterilir.
- Admin sayfası, yönetici olmayan kullanıcılar için tamamen render edilmez; `fetchMe()` rol kontrolü ile gizlenir.
- E-posta boş/geçersiz formatta → tarayıcı form doğrulaması istek öncesinde engeller; API `400` fallback.
- Aynı anda iki yönetici aynı e-postayı oluşturmaya çalışırsa DB uniqueness `409` üretir; UI'da çakışma mesajı gösterilir.

## Assumptions

- `POST /api/identity/users` M1 semantiğiyle kullanılır; şifre üretimi, gösterimi ve must-change akışı bu spec'in kapsamıdır.
- Liste uç noktası (`GET /api/identity/users`) bu modülle eklendi; pagination yok (tek instance, az kullanıcı), sıralama e-posta asc.
- UI dili Türkçe kalır (mevcut sayfalarla tutarlı).
- Migration gerekmez; users tablosu M1'den beri mevcut.
- Rol sabitleri: `admin` ve `member` — UI'da `role === "admin"` kontrolü kullanılır (mevcut `fetchMe` contract'ı korunur).

## Out of Scope

- Şifre sıfırlama/OTP akışı, e-posta ile davet, self-servis kayıt (PRD gereği var olmayacak).
- Rol değiştirme, hesap silme/devre dışı bırakma (operator CLI işleri M1'de kalır).
- Kullanıcı listesi için pagination/search.

## Clarifications

Yok.

## Deviations

Yok.
