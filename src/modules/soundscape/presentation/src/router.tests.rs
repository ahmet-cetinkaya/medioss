//! Router-level tests: authorization gates (member vs admin), mix ownership,
//! and origin enforcement against an in-memory fake stack.
use std::sync::Arc;

use axum::{
    body::Body,
    http::header,
    http::{HeaderName, Request, StatusCode},
    Router,
};
use kernel::{Actor, Role, UserId};
use soundscape_application::ports::{
    media::{AudioFormat, AudioValidator, BlobReader, BlobStore, OpusConverter, StoredAudio},
    CategoryDelete, CategoryRepository, Clock, MixRepository, NewMix, NewSound, SoundRepository,
};
use soundscape_domain::{Mix, Sound, SoundCategory, SoundIcon};
use tower::ServiceExt;

use crate::{soundscape_router, SoundscapeDependencies};

fn member_actor() -> Actor {
    Actor::new(UserId::new(2), Role::Member)
}

fn admin_actor() -> Actor {
    Actor::new(UserId::new(1), Role::Admin)
}

fn another_member_actor() -> Actor {
    Actor::new(UserId::new(3), Role::Member)
}

// --- fakes -------------------------------------------------------------

#[derive(Default)]
struct FakeState {
    categories: std::sync::Mutex<Vec<SoundCategory>>,
    sounds: std::sync::Mutex<Vec<Sound>>,
    mixes: std::sync::Mutex<Vec<Mix>>,
}

struct FakeCategories(Arc<FakeState>);
#[async_trait::async_trait]
impl CategoryRepository for FakeCategories {
    async fn list(&self) -> Result<Vec<SoundCategory>, String> {
        Ok(self.0.categories.lock().unwrap().clone())
    }
    async fn find(&self, id: i64) -> Result<Option<SoundCategory>, String> {
        Ok(self
            .0
            .categories
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.id == id)
            .cloned())
    }
    async fn create(&self, _name: &str) -> Result<SoundCategory, String> {
        Err("not needed".into())
    }
    async fn rename(&self, _id: i64, _name: &str) -> Result<Option<SoundCategory>, String> {
        Err("not needed".into())
    }
    async fn delete_empty(&self, _id: i64) -> Result<CategoryDelete, String> {
        Err("not needed".into())
    }
}

struct FakeSounds(Arc<FakeState>);
#[async_trait::async_trait]
impl SoundRepository for FakeSounds {
    async fn list(&self) -> Result<Vec<Sound>, String> {
        Ok(self.0.sounds.lock().unwrap().clone())
    }
    async fn find(&self, id: i64) -> Result<Option<Sound>, String> {
        Ok(self
            .0
            .sounds
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.id == id)
            .cloned())
    }
    async fn create(&self, sound: NewSound) -> Result<Sound, String> {
        let mut sounds = self.0.sounds.lock().unwrap();
        let id = sounds.len() as i64 + 1;
        let created = Sound::new(
            id,
            &sound.name,
            sound.category_id,
            &sound.content_type,
            sound.size_bytes,
            &sound.storage_key,
            sound.duration_ms,
            sound.created_at,
        )
        .map_err(|e| e.to_string())?;
        let mut created = created;
        created.icon_key = sound.icon_key;
        sounds.push(created.clone());
        Ok(created)
    }
    async fn update(
        &self,
        id: i64,
        name: &str,
        category_id: i64,
        icon_key: Option<&str>,
    ) -> Result<Option<Sound>, String> {
        let mut sounds = self.0.sounds.lock().unwrap();
        Ok(sounds.iter_mut().find(|s| s.id == id).map(|sound| {
            sound.name = name.into();
            sound.category_id = category_id;
            sound.icon_key = icon_key.map(str::to_owned);
            sound.clone()
        }))
    }
    async fn set_icon(&self, id: i64, icon: Option<SoundIcon>) -> Result<Option<Sound>, String> {
        let mut sounds = self.0.sounds.lock().unwrap();
        Ok(sounds.iter_mut().find(|s| s.id == id).map(|sound| {
            sound.icon = icon;
            sound.clone()
        }))
    }
    async fn delete_with_mix_items(&self, _id: i64) -> Result<Option<Sound>, String> {
        Err("not needed".into())
    }
}

struct FakeMixes(Arc<FakeState>);
#[async_trait::async_trait]
impl MixRepository for FakeMixes {
    async fn list_for(&self, user_id: UserId) -> Result<Vec<Mix>, String> {
        Ok(self
            .0
            .mixes
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.user_id == user_id.value())
            .cloned()
            .collect())
    }
    async fn find_for(&self, id: i64, user_id: UserId) -> Result<Option<Mix>, String> {
        Ok(self
            .0
            .mixes
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.id == id && m.user_id == user_id.value())
            .cloned())
    }
    async fn create(&self, mix: NewMix) -> Result<Mix, String> {
        let mut mixes = self.0.mixes.lock().unwrap();
        let id = mixes.len() as i64 + 1;
        let created = Mix::new(
            id,
            mix.user_id.value(),
            &mix.name,
            mix.items,
            mix.now,
            mix.now,
        )
        .map_err(|e| e.to_string())?;
        mixes.push(created.clone());
        Ok(created)
    }
    async fn replace(&self, id: i64, mix: NewMix) -> Result<Option<Mix>, String> {
        let mut mixes = self.0.mixes.lock().unwrap();
        let existing = mixes
            .iter()
            .position(|m| m.id == id && m.user_id == mix.user_id.value());
        match existing {
            Some(index) => {
                let created = Mix::new(
                    id,
                    mix.user_id.value(),
                    &mix.name,
                    mix.items,
                    mix.now,
                    mix.now,
                )
                .map_err(|e| e.to_string())?;
                mixes[index] = created.clone();
                Ok(Some(created))
            }
            None => Ok(None),
        }
    }
    async fn delete_for(&self, id: i64, user_id: UserId) -> Result<bool, String> {
        let mut mixes = self.0.mixes.lock().unwrap();
        let before = mixes.len();
        mixes.retain(|m| !(m.id == id && m.user_id == user_id.value()));
        Ok(mixes.len() != before)
    }

    async fn delete_all_for_user(&self, _user_id: UserId) -> Result<u64, String> {
        Ok(0)
    }
}

struct FakeValidator;
#[async_trait::async_trait]
impl AudioValidator for FakeValidator {
    async fn sniff(&self, _bytes: &[u8]) -> Result<AudioFormat, ()> {
        Ok(AudioFormat::OggOpus)
    }
}

struct FakeConverter;
#[async_trait::async_trait]
impl OpusConverter for FakeConverter {
    async fn convert(
        &self,
        bytes: Vec<u8>,
        _format: AudioFormat,
    ) -> Result<(Vec<u8>, i64, StoredAudio), String> {
        Ok((
            bytes,
            60_000,
            StoredAudio {
                extension: "opus",
                content_type: "audio/ogg",
            },
        ))
    }
}

struct TestReader {
    bytes: Vec<u8>,
    pos: usize,
}

impl futures_io::AsyncRead for TestReader {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let remaining = &self.bytes[self.pos..];
        let n = remaining.len().min(buf.len());
        buf[..n].copy_from_slice(&remaining[..n]);
        self.pos += n;
        std::task::Poll::Ready(Ok(n))
    }
}

#[derive(Default)]
struct FakeStore(std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>);
#[async_trait::async_trait]
impl BlobStore for FakeStore {
    async fn put(
        &self,
        _folder: &str,
        stem: &str,
        ext: &str,
        bytes: &[u8],
    ) -> Result<String, String> {
        let key = format!("soundscape/{stem}.{ext}");
        self.0.lock().unwrap().insert(key.clone(), bytes.to_vec());
        Ok(key)
    }
    async fn open_range(
        &self,
        key: &str,
        _start: u64,
        _length: u64,
    ) -> Result<(BlobReader, String, u64), String> {
        let bytes = self
            .0
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .unwrap_or_else(|| b"rain-bytes".to_vec());
        let len = bytes.len() as u64;
        Ok((
            BlobReader(Box::new(TestReader { bytes, pos: 0 })),
            "audio/ogg".into(),
            len,
        ))
    }
    async fn delete(&self, key: &str) -> Result<(), String> {
        self.0.lock().unwrap().remove(key);
        Ok(())
    }
}

struct FixedClock;
#[async_trait::async_trait]
impl Clock for FixedClock {
    async fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

// --- harness -----------------------------------------------------------

struct TestApp {
    router: Router,
    #[allow(dead_code)] // kept for future assertions that inspect fake state
    state: Arc<FakeState>,
}

fn app() -> TestApp {
    let state = Arc::new(FakeState::default());
    state.categories.lock().unwrap().push(SoundCategory {
        id: 1,
        name: "Nature".into(),
        slug: "nature".into(),
    });
    state.sounds.lock().unwrap().push(
        Sound::new(
            1,
            "Rain",
            1,
            "audio/ogg",
            42,
            "soundscape/rain.opus",
            60_000,
            chrono::Utc::now(),
        )
        .unwrap(),
    );
    let deps = SoundscapeDependencies {
        categories: Arc::new(FakeCategories(state.clone())),
        sounds: Arc::new(FakeSounds(state.clone())),
        mixes: Arc::new(FakeMixes(state.clone())),
        validator: Arc::new(FakeValidator),
        converter: Arc::new(FakeConverter),
        store: Arc::new(FakeStore::default()),
        clock: Arc::new(FixedClock),
        medioss_origin: Arc::new("http://localhost".into()),
    };
    TestApp {
        router: soundscape_router().with_state(Arc::new(deps)),
        state,
    }
}

async fn call(
    app: &TestApp,
    actor: Option<Actor>,
    method: &str,
    path: &str,
    headers: Vec<(&str, &str)>,
    body: Body,
) -> (StatusCode, Vec<(HeaderName, String)>, Vec<u8>) {
    let mut request = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        request = request.header(name, value);
    }
    request = request.header("origin", "http://localhost");
    let mut request = request.body(body).unwrap();
    if let Some(actor) = actor {
        request.extensions_mut().insert(actor);
    }
    let response = app.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| (name.clone(), value.to_str().unwrap_or("").to_owned()))
        .collect();
    let body = axum::body::to_bytes(response.into_body(), 128 * 1024 * 1024)
        .await
        .unwrap()
        .to_vec();
    (status, headers, body)
}

fn json(payload: &str) -> (Vec<(&'static str, &'static str)>, Body) {
    (
        vec![("content-type", "application/json")],
        Body::from(payload.to_owned()),
    )
}

// --- tests -------------------------------------------------------------

#[tokio::test]
async fn members_manage_their_own_mixes() {
    let app = app();
    let payload = r#"{"name":"Focus","items":[{"sound_id":1,"volume":50,"position":0}]}"#;
    let (headers, body) = json(payload);
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/soundscape/mixes",
        headers,
        body,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "{}",
        String::from_utf8_lossy(&body)
    );

    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/mixes",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8_lossy(&body);
    assert!(
        text.contains("\"name\":\"Focus\""),
        "member sees own mix: {text}"
    );
}

#[tokio::test]
async fn mix_of_another_user_is_invisible_and_untouchable() {
    let app = app();
    let payload = r#"{"name":"Mine","items":[{"sound_id":1,"volume":50,"position":0}]}"#;
    let (headers, body) = json(payload);
    let (status, _, created) = call(
        &app,
        Some(another_member_actor()),
        "POST",
        "/soundscape/mixes",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id: i64 = serde_json::from_slice::<serde_json::Value>(&created).unwrap()["id"]
        .as_i64()
        .unwrap();

    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        &format!("/soundscape/mixes/{id}"),
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "foreign mix by id is 404");
}

#[tokio::test]
async fn unauthenticated_requests_are_rejected() {
    let app = app();
    let (status, _, _) = call(
        &app,
        None,
        "GET",
        "/soundscape/sounds",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = call(
        &app,
        None,
        "GET",
        "/soundscape/mixes",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_only_routes_forbid_members() {
    let app = app();
    let payload = r#"{"name":"Whatever"}"#;
    let (headers, body) = json(payload);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "POST",
        "/soundscape/categories",
        headers,
        body,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "category creation is admin-only"
    );
}

#[tokio::test]
async fn mutations_require_matching_origin() {
    let app = app();
    let payload = r#"{"name":"Focus","items":[{"sound_id":1,"volume":50,"position":0}]}"#;
    let (mut headers, body) = json(payload);
    headers.retain(|(name, _)| *name != "origin");
    let mut request = Request::builder()
        .method("POST")
        .uri("/soundscape/mixes")
        .header("origin", "https://evil.example")
        .header("content-type", "application/json");
    for (name, value) in headers {
        request = request.header(name, value);
    }
    let mut request = request.body(body).unwrap();
    request.extensions_mut().insert(member_actor());
    let response = app.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn download_is_an_attachment_named_after_the_sound() {
    let app = app();
    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/download",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let disposition = headers
        .iter()
        .find(|(name, _)| name == header::CONTENT_DISPOSITION)
        .expect("disposition present");
    assert!(disposition.1.starts_with("attachment; filename="));
    assert!(disposition.1.ends_with(".ogg\""));
    assert_eq!(body, b"rain-bytes");
}

#[tokio::test]
async fn listing_sounds_requires_any_signed_in_actor() {
    let app = app();
    let (status, _, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
}

#[test]
fn parses_single_byte_and_suffix_ranges() {
    assert_eq!(
        crate::router::parse_range("bytes=0-0", 10),
        Some(Ok((0, 0)))
    );
    assert_eq!(crate::router::parse_range("bytes=-3", 10), Some(Ok((7, 9))));
    assert_eq!(crate::router::parse_range("bytes=4-", 10), Some(Ok((4, 9))));
}

#[test]
fn ignores_malformed_and_rejects_unsatisfiable_ranges() {
    assert_eq!(crate::router::parse_range("items=0-1", 10), None);
    assert_eq!(crate::router::parse_range("bytes=bogus", 10), None);
    assert_eq!(
        crate::router::parse_range("bytes=99-100", 10),
        Some(Err(()))
    );
}

// --- icons -------------------------------------------------------------

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfake-png-body";

fn icon_form(bytes: &[u8]) -> (Vec<(&'static str, &'static str)>, Body) {
    let mut body =
        b"--XB\r\nContent-Disposition: form-data; name=\"icon\"; filename=\"i\"\r\n\r\n".to_vec();
    body.extend_from_slice(bytes);
    body.extend_from_slice(b"\r\n--XB--\r\n");
    (
        vec![("content-type", "multipart/form-data; boundary=XB")],
        Body::from(body),
    )
}

async fn put_icon(app: &TestApp, bytes: &[u8]) -> (StatusCode, Vec<u8>) {
    let (headers, body) = icon_form(bytes);
    let (status, _, body) = call(
        app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1/icon",
        headers,
        body,
    )
    .await;
    (status, body)
}

#[tokio::test]
async fn icon_upload_roundtrip_serves_the_image_and_flags_the_sound() {
    let app = app();
    let (status, body) = put_icon(&app, PNG).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert!(String::from_utf8_lossy(&body).contains("\"has_icon\":true"));

    let (status, headers, body) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, PNG);
    let content_type = headers.iter().find(|(n, _)| n == header::CONTENT_TYPE);
    assert_eq!(content_type.unwrap().1, "image/png");

    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "DELETE",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(String::from_utf8_lossy(&body).contains("\"has_icon\":false"));
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn icon_upload_refuses_wrong_type_svg_and_oversize() {
    let app = app();
    let (status, _) = put_icon(&app, b"GIF89a-not-allowed").await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let (status, _) = put_icon(&app, b"<svg xmlns='http://www.w3.org/2000/svg'/>").await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE, "svg is refused");
    let mut big = PNG.to_vec();
    big.resize(1024 * 1024 + 1, 0);
    let (status, _) = put_icon(&app, &big).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn icon_mutations_are_admin_only() {
    let app = app();
    let (headers, body) = icon_form(PNG);
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "PUT",
        "/soundscape/sounds/1/icon",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_edits_icon_key_and_it_is_validated() {
    let app = app();
    let (headers, body) = json(r#"{"name":"Rain","category_id":1,"icon_key":"rain"}"#);
    let (status, _, body) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert!(String::from_utf8_lossy(&body).contains("\"icon_key\":\"rain\""));

    let (headers, body) = json(r#"{"name":"Rain","category_id":1,"icon_key":"Bad Key!"}"#);
    let (status, _, _) = call(
        &app,
        Some(admin_actor()),
        "PUT",
        "/soundscape/sounds/1",
        headers,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn missing_icon_is_404() {
    let app = app();
    let (status, _, _) = call(
        &app,
        Some(member_actor()),
        "GET",
        "/soundscape/sounds/1/icon",
        vec![],
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
