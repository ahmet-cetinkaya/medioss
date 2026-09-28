---
status: approved
capability: identity
plan:
supersedes:
---

# Kullanıcı Yönetimi — Yaşayan Spec Deltası

## Intent

Yönetici (admin) kimliği doğrulanmış kullanıcıları, uygulamanın hiçbir yerine dokunmadan bir ekran üzerinden ekleyebilsin. Bu değişiklik, M1 kimlik modülünün mevcut davranışını değiştirmez; API yüzeyi (`POST /api/identity/users`, FR-013) zaten mevcut ve testli olduğundan, eksik kalan tek parça kimliği doğrulanmış yönetici için web UI yüzeyidir. Bu spec, o UI yüzeyinin davranış sözleşmesini belirler ve tamamlanmasıyla birlikte M1 spec'inde takip edilen US5/FR-013 kullanıcı hikâyesi gerekliliklerini uçtan uca karşılanmış sayar.

## User Stories

### US1 — Yönetici, kullanıcı ekler (Priority: Kritik)

**Bireysel test:** Yönetici olarak admin sayfasını aç; e-posta ve geçici şifre girip gönder; yeni kullanıcı listede görünür ve oluşturulan hesapla ilk girişte şifre değişikliği istenir.

- **Given** yönetici oturumu ve kullanılmayan bir e-posta, **when** kullanıcı oluşturulursa, **then** istek `201` döner, hesap `user`/member rolüyle ve must-change-password işaretiyle oluşur, ve UI tek kullanımlık şifreyi bir kez, yalnızca bu ekranda gösterir.
- **Given** zaten mevcut bir e-posta (büyük/küçük harf farkı dahil), **when** kullanıcı oluşturulursa, **then** UI `409` hatasını aynı dilde bir hata mesajı olarak gösterir ve form temizlenmez.
- **Given** 8 karakterden kısa bir geçici şifre, **when** form gönderilirse, **then** API `400` döner ve UI doğrulama hatasını alanın altında gösterir.
- **Given** üye oturumu, **when** admin sayfası açılmaya çalışılırsa, **then** sayfa yönetici olmayanlara içeriği göstermez ve yönetim aksiyonları 403 ile reddedilir.

### US2 — Yönetici, kullanıcıları listeler (Priority: Düşük)

**Bireysel test:** Yönetici olarak admin sayfasını aç; hesap listesi e-posta, rol ve oluşturma zamanıyla görüntülenir.

- **Given** yönetici oturumu, **when** admin sayfası açılır, **then** mevcut hesaplar (id, e-posta, rol) listelenir.
- **Given** üye oturumu, **when** hesap listesi istenirse, **then** istek 403 ile reddedilir.

## Requirements

### ADDED

- FR-022: WHEN yönetici oturumlu bir istemci admin sayfasındaki formu kullanılmayan bir e-posta ve en az 8 karakterlik bir geçici şifreyle gönderirse THE SYSTEM SHALL `201` ile hesabı oluşturup UI'da listeye eklesin; hesap member rolünde ve must-change-password işaretli doğar.
- FR-023: WHEN oluşturma isteği `409` (e-posta çakışması) veya `400` (doğrulama) döndürürse THE SYSTEM SHALL formdaki girişleri koruyup mesajı formun yanında aynı dilde göstersin.
- FR-024: WHEN üye rolündeki bir kullanıcı admin sayfasına erişmeye çalışırsa THE SYSTEM SHALL sayfayı boş/erişilemez gösterip API çağrılarını yapmadan reddetsin (API zaten 403 döner; UI kaba dokunmadan gizler).
- FR-025: WHEN admin sayfası yüklendiğinde THE SYSTEM SHALL mevcut kullanıcıları (id, e-posta, rol, oluşturma zamanı) çekebilsin; liste uç noktası bu değişiklikle eklenir ve yalnızca yönetici erişebilir.
- FR-025a: WHEN geçici şifre gösterim alanı kullanılırsa THE SYSTEM SHALL şifreyi yalnızca oluşturma anında bir kez görüntüleyip sonraki render'larda ve sayfa yenilemelerinde erişilemez kılsın; şifre yöneticinin ekranında kalıcı olarak saklanmaz.

### PRESERVED

- FR-013 (ADDED → PRESERVED): Admin API kullanıcı oluşturma semantiği — unused e-posta → 201/member; duplicate (case-insensitive) → 409; non-admin → 403 — değişmeden kalır ve UI bu API'yi çağırır.
- FR-014 (ADDED → PRESERVED): E-posta normalizasyonu ve veritabanı seviyesinde case-insensitive uniqueness aynen korunur.
- FR-012 (ADDED → YENİDEN TEYİT): Geçici şifre en az 8 karakter kuralı UI'da da aynı doğrulama mesajıyla yansıtılır.
- FR-004/FR-018 (PRESERVED): Oturum döndürme ve argon2 kimlik doğrulama davranışları bu değişiklikten etkilenmez.

### REMOVED

- Yok.

## Success Criteria

- SC-001: Yönetici, admin sayfasından bir kullanıcı oluşturur; istek `201`, hesap listede, yeni kullanıcı ilk girişte şifre değişikliğine zorlanır (canlı QA: admin ekler → üye geçici şifreyle girer → şifre değişimi istenir).
- SC-002: Aynı e-posta tekrar (farklı case) gönderildiğinde UI çakışma hatası gösterir, form korunur; API seviyesinde `409` doğrulanır.
- SC-003: Üye oturumuyla admin sayfası açıldığında kullanıcı listesi çekilmez ve oluşturma aksiyonu `403` alır; rota testleriyle kanıtlanır.
- SC-004: Rota seviyesi testler: admin `POST /api/identity/users` → 201; üye → 403; anonim → 401; `GET /api/identity/users` (yeni) → admin 200/üye 403.
- SC-004a: Yeni kullanıcının ilk girişinde must-change-password akışı uygundur: geçici şifreyle login başarılı ve `me` payload'ında `must_change_password: true` döner.
- SC-005: `/ac:spec` kabulünden sonra `cargo test --workspace` ve UI build başarılı; canlı doğrulama tamamlanır.

## Edge Cases

- Geçici şifre, yönetici e-posta ile değil yalnızca ekranda gösterilir; sayfa yenilendiğinde tekrar görünmez.
- E-posta büyük/küçük harf farklarıyla girilirse, API normalizasyonuyla `409` tetiklenir; UI mesajı hem 409 hem 400 için aynı formda gösterilir.
- Admin sayfası, yönetici olmayan kullanıcılar için tamamen render edilmez; `fetchMe()` rol kontrolü ile gizlenir.
- E-posta boş/geçersiz formatta → API 400; UI alan-doğrulama hatası gösterir, istek gönderilmez (tarayıcı form doğrulaması + API 400 fallback).
- Aynı anda iki yönetici aynı e-postayı oluşturmaya çalışırsa DB uniqueness `409` üretir; UI'da çakışma mesajı gösterilir.

## Assumptions

- `POST /api/identity/users` mevcut semantiğiyle kullanılır; şifre üretimi, gösterimi ve must-change akışı bu spec'in kapsamıdır.
- Liste uç noktası (`GET /api/identity/users`) yeni eklenir; pagination yok (tek instance, az kullanıcı), sıralama e-posta asc.
- UI dili Türkçe kalır (mevcut sayfalarla tutarlı).
- Yeni migration gerekmez; users tablosu zaten mevcut.
- Role sabitleri: `admin` ve `user`(member) — UI'da `role === "admin"` kontrolü kullanılır (mevcut `fetchMe` contract'ı korunur).

## Out of Scope

- Şifre sıfırlama/OTP akışı, e-posta ile davet, self-servis kayıt (PRD gereği var olmayacak).
- Rol değiştirme, hesap silme/devre dışı bırakma (kullanıcı yoksa throttle clear gibi operator CLI işleri kalır).
- Kullanıcı listesi için pagination/search.

## Clarifications

Yok — varsayım kabulüne kadar.
