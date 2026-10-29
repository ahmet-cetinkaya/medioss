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
    staged_upload::StagedMedia,
    CategoryDelete, CategoryRepository, Clock, MixRepository, NewMix, NewSound, SoundRepository,
};
use soundscape_application::test_support::FakeUploadStaging;
use soundscape_domain::{Mix, Sound, SoundCategory, SoundIcon};
use tower::ServiceExt;

use crate::router::{soundscape_router, SoundscapeDependencies};

pub(super) const TEST_MAX_UPLOAD_BYTES: usize = 100 * 1024 * 1024;

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
    pub(super) staging: FakeUploadStaging,
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
    async fn create(&self, name: &str) -> Result<SoundCategory, String> {
        let mut categories = self.0.categories.lock().unwrap();
        if categories.iter().any(|c| c.name.eq_ignore_ascii_case(name)) {
            return Err("UNIQUE constraint failed: soundscape_categories.name".into());
        }
        let category = SoundCategory {
            id: categories.iter().map(|c| c.id).max().unwrap_or(0) + 1,
            name: name.trim().into(),
            slug: name.trim().to_lowercase(),
        };
        categories.push(category.clone());
        Ok(category)
    }
    async fn rename(&self, id: i64, name: &str) -> Result<Option<SoundCategory>, String> {
        let mut categories = self.0.categories.lock().unwrap();
        Ok(categories.iter_mut().find(|c| c.id == id).map(|category| {
            category.name = name.trim().into();
            category.slug = name.trim().to_lowercase();
            category.clone()
        }))
    }
    async fn delete_empty(&self, id: i64) -> Result<CategoryDelete, String> {
        if self
            .0
            .sounds
            .lock()
            .unwrap()
            .iter()
            .any(|s| s.category_id == id)
        {
            return Ok(CategoryDelete::InUse);
        }
        let mut categories = self.0.categories.lock().unwrap();
        let before = categories.len();
        categories.retain(|c| c.id != id);
        Ok(if categories.len() == before {
            CategoryDelete::NotFound
        } else {
            CategoryDelete::Deleted
        })
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
    async fn delete_with_mix_items(&self, id: i64) -> Result<Option<Sound>, String> {
        let mut sounds = self.0.sounds.lock().unwrap();
        Ok(sounds
            .iter()
            .position(|s| s.id == id)
            .map(|index| sounds.remove(index)))
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
        media: StagedMedia,
        _format: AudioFormat,
    ) -> Result<(StagedMedia, i64, StoredAudio), String> {
        Ok((
            media,
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
    async fn put_staged(
        &self,
        media: StagedMedia,
        _folder: &str,
        stem: &str,
        ext: &str,
    ) -> Result<String, String> {
        let bytes = self.0.staging.take(&media).ok_or("staged file missing")?;
        self.put("soundscape", stem, ext, &bytes).await
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
    pub(super) state: Arc<FakeState>,
}

pub(super) fn app() -> TestApp {
    app_with_cap(TEST_MAX_UPLOAD_BYTES)
}

/// An app whose upload cap is `max_upload_bytes`, so a short body can overflow it.
pub(super) fn app_with_cap(max_upload_bytes: usize) -> TestApp {
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
    let categories: Arc<dyn CategoryRepository> = Arc::new(FakeCategories(state.clone()));
    let sounds: Arc<dyn SoundRepository> = Arc::new(FakeSounds(state.clone()));
    let mixes: Arc<dyn MixRepository> = Arc::new(FakeMixes(state.clone()));
    let store: Arc<dyn BlobStore> = Arc::new(FakeStore(state.clone()));
    let deps = SoundscapeDependencies {
        list_categories: Arc::new(
            soundscape_application::queries::list_categories::ListCategoriesQueryHandler::new(
                categories.clone(),
            ),
        ),
        list_sounds: Arc::new(
            soundscape_application::queries::list_sounds::ListSoundsQueryHandler::new(
                sounds.clone(),
            ),
        ),
        find_sound_icon: Arc::new(
            soundscape_application::queries::find_sound_icon::FindSoundIconQueryHandler::new(
                sounds.clone(),
            ),
        ),
        create_category: Arc::new(
            soundscape_application::commands::create_category::CreateCategoryCommandHandler::new(
                categories.clone(),
            ),
        ),
        rename_category: Arc::new(
            soundscape_application::commands::rename_category::RenameCategoryCommandHandler::new(
                categories.clone(),
            ),
        ),
        delete_category: Arc::new(
            soundscape_application::commands::delete_category::DeleteCategoryCommandHandler::new(
                categories.clone(),
            ),
        ),
        create_sound: Arc::new(
            soundscape_application::commands::create_sound::CreateSoundCommandHandler::new(
                categories.clone(),
                sounds.clone(),
                Arc::new(FakeValidator),
                Arc::new(FakeConverter),
                store.clone(),
                Arc::new(state.staging.clone()),
                Arc::new(FakeImageNormalizer(state.clone())),
                max_upload_bytes,
            ),
        ),
        update_sound: Arc::new(
            soundscape_application::commands::update_sound::UpdateSoundCommandHandler::new(
                categories.clone(),
                sounds.clone(),
            ),
        ),
        delete_sound: Arc::new(
            soundscape_application::commands::delete_sound::DeleteSoundCommandHandler::new(
                sounds.clone(),
                store.clone(),
            ),
        ),
        set_sound_icon: Arc::new(
            soundscape_application::commands::set_sound_icon::SetSoundIconCommandHandler::new(
                sounds.clone(),
                store.clone(),
                Arc::new(FakeImageNormalizer(state.clone())),
            ),
        ),
        clear_sound_icon: Arc::new(
            soundscape_application::commands::clear_sound_icon::ClearSoundIconCommandHandler::new(
                sounds.clone(),
                store.clone(),
            ),
        ),
        create_mix: Arc::new(
            soundscape_application::commands::create_mix::CreateMixCommandHandler::new(
                mixes.clone(),
                sounds.clone(),
                Arc::new(FixedClock),
            ),
        ),
        update_mix: Arc::new(
            soundscape_application::commands::update_mix::UpdateMixCommandHandler::new(
                mixes.clone(),
                sounds.clone(),
                Arc::new(FixedClock),
            ),
        ),
        delete_mix: Arc::new(
            soundscape_application::commands::delete_mix::DeleteMixCommandHandler::new(
                mixes.clone(),
            ),
        ),
        store,
        sounds,
        mixes,
        medioss_origin: Arc::new("http://localhost".into()),
        staging: Arc::new(state.staging.clone()),
        max_upload_bytes,
    };
    TestApp {
        router: soundscape_router(max_upload_bytes).with_state(Arc::new(deps)),
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

pub(super) fn header_value(headers: &[(HeaderName, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.as_str() == name)
        .map(|(_, value)| value.clone())
}
