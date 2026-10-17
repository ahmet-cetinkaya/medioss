use std::sync::Arc;

use axum::{
    body::Body,
    http::{HeaderName, Request, StatusCode},
    Router,
};
use kernel::{Actor, Role, UserId};
use soundscape_application::ports::{
    media::{
        AudioFormat, AudioValidator, BlobReader, BlobStore, ImageNormalizationError,
        ImageNormalizer, OpusConverter, StoredAudio,
    },
    CategoryDelete, CategoryRepository, Clock, MixRepository, NewMix, NewSound, SoundRepository,
};
use soundscape_domain::{Mix, Sound, SoundCategory, SoundIcon};
use tower::ServiceExt;

use crate::{soundscape_router, SoundscapeDependencies};

pub(super) fn member_actor() -> Actor {
    Actor::new(UserId::new(2), Role::Member)
}

pub(super) fn admin_actor() -> Actor {
    Actor::new(UserId::new(1), Role::Admin)
}

pub(super) fn another_member_actor() -> Actor {
    Actor::new(UserId::new(3), Role::Member)
}

// --- fakes -------------------------------------------------------------

#[derive(Default)]
pub(super) struct FakeState {
    pub(super) categories: std::sync::Mutex<Vec<SoundCategory>>,
    pub(super) sounds: std::sync::Mutex<Vec<Sound>>,
    pub(super) mixes: std::sync::Mutex<Vec<Mix>>,
    pub(super) store: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
    pub(super) normalization_count: std::sync::Mutex<usize>,
    pub(super) normalization_error: std::sync::Mutex<Option<ImageNormalizationError>>,
    pub(super) open_error: std::sync::Mutex<Option<String>>,
}

pub(super) struct FakeCategories(pub(super) Arc<FakeState>);
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

pub(super) struct FakeSounds(pub(super) Arc<FakeState>);
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

pub(super) struct FakeMixes(pub(super) Arc<FakeState>);
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

pub(super) struct FakeImageNormalizer(pub(super) Arc<FakeState>);
#[async_trait::async_trait]
impl ImageNormalizer for FakeImageNormalizer {
    async fn normalize(&self, _bytes: Vec<u8>) -> Result<Vec<u8>, ImageNormalizationError> {
        *self.0.normalization_count.lock().unwrap() += 1;
        if let Some(error) = *self.0.normalization_error.lock().unwrap() {
            return Err(error);
        }
        Ok(b"RIFF\x04\0\0\0WEBP".to_vec())
    }
}

pub(super) struct FakeValidator;
#[async_trait::async_trait]
impl AudioValidator for FakeValidator {
    async fn sniff(&self, _bytes: &[u8]) -> Result<AudioFormat, ()> {
        Ok(AudioFormat::OggOpus)
    }
}

pub(super) struct FakeConverter;
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

pub(super) struct TestReader {
    pub(super) bytes: Vec<u8>,
    pub(super) pos: usize,
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
pub(super) struct FakeStore(pub(super) std::sync::Arc<FakeState>);
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
        self.0
            .store
            .lock()
            .unwrap()
            .insert(key.clone(), bytes.to_vec());
        Ok(key)
    }
    async fn open_range(
        &self,
        key: &str,
        _start: u64,
        // Deliberately ignores the requested length, like a store whose blob
        // changed out-of-band: the reader must bound itself.
        _length: u64,
    ) -> Result<(BlobReader, String, u64), String> {
        if let Some(error) = self.0.open_error.lock().unwrap().clone() {
            return Err(error);
        }
        let bytes = self
            .0
            .store
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
        self.0.store.lock().unwrap().remove(key);
        Ok(())
    }
}

pub(super) struct FixedClock;
#[async_trait::async_trait]
impl Clock for FixedClock {
    async fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

// --- harness -----------------------------------------------------------

pub(super) struct TestApp {
    pub(super) router: Router,
    #[allow(dead_code)] // kept for future assertions that inspect fake state
    pub(super) state: Arc<FakeState>,
}

pub(super) fn app() -> TestApp {
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
        store: Arc::new(FakeStore(state.clone())),
        image_normalizer: Arc::new(FakeImageNormalizer(state.clone())),
        clock: Arc::new(FixedClock),
        medioss_origin: Arc::new("http://localhost".into()),
    };
    TestApp {
        router: soundscape_router().with_state(Arc::new(deps)),
        state,
    }
}

pub(super) async fn call(
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

pub(super) fn json(payload: &str) -> (Vec<(&'static str, &'static str)>, Body) {
    (
        vec![("content-type", "application/json")],
        Body::from(payload.to_owned()),
    )
}

// --- tests -------------------------------------------------------------

// --- icons -------------------------------------------------------------

pub(super) const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfake-png-body";

pub(super) fn icon_form(bytes: &[u8]) -> (Vec<(&'static str, &'static str)>, Body) {
    let mut body =
        b"--XB\r\nContent-Disposition: form-data; name=\"icon\"; filename=\"i\"\r\n\r\n".to_vec();
    body.extend_from_slice(bytes);
    body.extend_from_slice(b"\r\n--XB--\r\n");
    (
        vec![("content-type", "multipart/form-data; boundary=XB")],
        Body::from(body),
    )
}

pub(super) async fn put_icon(app: &TestApp, bytes: &[u8]) -> (StatusCode, Vec<u8>) {
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

pub(super) fn sound_form(icon: &[u8]) -> Body {
    let mut body = b"--XB\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\nRain\r\n--XB\r\nContent-Disposition: form-data; name=\"category_id\"\r\n\r\n1\r\n--XB\r\nContent-Disposition: form-data; name=\"file\"; filename=\"rain.ogg\"\r\n\r\naudio\r\n--XB\r\nContent-Disposition: form-data; name=\"icon\"; filename=\"rain.png\"\r\n\r\n".to_vec();
    body.extend_from_slice(icon);
    body.extend_from_slice(b"\r\n--XB--\r\n");
    Body::from(body)
}

pub(super) async fn post_sound(app: &TestApp, icon: &[u8]) -> (StatusCode, Vec<u8>) {
    let (status, _, body) = call(
        app,
        Some(admin_actor()),
        "POST",
        "/soundscape/sounds",
        vec![("content-type", "multipart/form-data; boundary=XB")],
        sound_form(icon),
    )
    .await;
    (status, body)
}
