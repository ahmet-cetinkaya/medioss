//! Test fakes for the meditation application ports (identity test_support
//! style: plain state behind `Arc<Mutex<..>>`, one fake per port).
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use kernel::UserId;

use crate::ports::{
    AudioFormat, AudioValidator, BlobStore, Category, CategoryRepository, Clock, CoverValidator,
    ImageFormat, MeditationSession, Membership, MembershipRepository, Note, NoteRepository,
    PlaybackState, PlaybackStateRepository, Program, ProgramRepository, SessionRepository,
    SniffError, StoredBlob,
};

/// A clock returning a fixed instant.
#[derive(Clone)]
pub struct FixedClock(pub DateTime<Utc>);

#[async_trait::async_trait]
impl Clock for FixedClock {
    async fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

/// Accepts any leading bytes that start with a known magic; rejects the rest.
/// Magic rules mirror the production sniffers and are the test oracle for
/// format detection without ffmpeg.
pub struct FakeValidator;

#[async_trait::async_trait]
impl AudioValidator for FakeValidator {
    async fn sniff(&self, leading: &[u8]) -> Result<AudioFormat, SniffError> {
        sniff_audio(leading)
    }
}

/// Production-shaped audio sniffing, shared by the fake and tests.
pub fn sniff_audio(bytes: &[u8]) -> Result<AudioFormat, SniffError> {
    if bytes.len() < 4 {
        return Err(SniffError::TooShort);
    }
    if bytes.starts_with(b"OggS") {
        let head = &bytes[..bytes.len().min(64)];
        return if head.windows(8).any(|window| window == b"OpusHead") {
            Ok(AudioFormat::OggOpus)
        } else {
            Ok(AudioFormat::OggVorbis)
        };
    }
    if bytes.starts_with(b"ID3") {
        return Ok(AudioFormat::Mp3);
    }
    if bytes[0] == 0xFF && bytes[1] & 0xE0 == 0xE0 {
        return Ok(AudioFormat::Mp3);
    }
    if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WAVE" {
        return Ok(AudioFormat::Wav);
    }
    Err(SniffError::Unsupported)
}

/// Accepts PNG/JPEG/WebP magics.
pub struct FakeCoverValidator;

#[async_trait::async_trait]
impl CoverValidator for FakeCoverValidator {
    async fn sniff(&self, leading: &[u8]) -> Result<ImageFormat, SniffError> {
        if leading.len() < 12 {
            return Err(SniffError::TooShort);
        }
        if leading.starts_with(&[0x89, b'P', b'N', b'G']) {
            return Ok(ImageFormat::Png);
        }
        if leading.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(ImageFormat::Jpeg);
        }
        if leading.starts_with(b"RIFF") && &leading[8..12] == b"WEBP" {
            return Ok(ImageFormat::Webp);
        }
        Err(SniffError::Unsupported)
    }
}

/// Passes Ogg Opus through unchanged; rewrites other formats to a fake
/// "encoded" payload so tests can observe the conversion call.
#[derive(Clone)]
pub struct FakeConverter {
    /// When set, every conversion fails (simulates a decode error).
    pub fail: bool,
}

#[async_trait::async_trait]
impl crate::ports::OpusConverter for FakeConverter {
    async fn convert(&self, bytes: Vec<u8>, format: AudioFormat) -> Result<Vec<u8>, String> {
        if self.fail {
            return Err("conversion failed".to_owned());
        }
        if format == AudioFormat::OggOpus {
            return Ok(bytes);
        }
        let mut encoded = b"OggS".to_vec();
        encoded.extend_from_slice(b"fake-opus-payload");
        Ok(encoded)
    }
}

/// In-memory blob store recording every put and delete.
#[derive(Clone, Default)]
pub struct FakeBlobStore {
    state: Arc<Mutex<std::collections::HashMap<String, Vec<u8>>>>,
    counter: Arc<Mutex<u64>>,
    pub failures: Arc<Mutex<bool>>,
}

impl FakeBlobStore {
    /// A store that starts empty and succeeds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored blobs.
    pub fn len(&self) -> usize {
        self.state.lock().unwrap().len()
    }

    /// Whether the store holds no blobs.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait::async_trait]
impl BlobStore for FakeBlobStore {
    async fn put(&self, bytes: &[u8]) -> Result<StoredBlob, String> {
        if *self.failures.lock().unwrap() {
            return Err("store down".to_owned());
        }
        let key = format!("key{}", {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            *counter
        });
        self.state
            .lock()
            .unwrap()
            .insert(key.clone(), bytes.to_vec());
        Ok(StoredBlob {
            storage_key: key,
            size_bytes: bytes.len() as i64,
        })
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        if *self.failures.lock().unwrap() {
            return Err("store down".to_owned());
        }
        self.state.lock().unwrap().remove(key);
        Ok(())
    }
}

/// Shared in-memory state for all repository fakes; clone gives another
/// handle to the same data.
#[derive(Clone, Default)]
pub struct RepoState {
    pub meditations: Arc<Mutex<Vec<meditation_domain::Meditation>>>,
    pub programs: Arc<Mutex<Vec<Program>>>,
    pub categories: Arc<Mutex<Vec<Category>>>,
    pub memberships: Arc<Mutex<Vec<Membership>>>,
    pub sessions: Arc<Mutex<Vec<MeditationSession>>>,
    pub notes: Arc<Mutex<Vec<Note>>>,
    pub playback: Arc<Mutex<std::collections::HashMap<(i64, i64), PlaybackState>>>,
}

impl RepoState {
    /// Fresh empty state.
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn next_id<T>(rows: &[T], id_of: impl Fn(&T) -> i64) -> i64 {
        rows.iter().map(id_of).max().unwrap_or(0) + 1
    }

    /// Inserts a ready-made meditation row (bypasses validation).
    #[allow(dead_code)]
    pub fn seed_meditation(&self, meditation: meditation_domain::Meditation) {
        self.meditations.lock().unwrap().push(meditation);
    }

    /// Directly sets a playback state row.
    #[allow(dead_code)]
    pub fn seed_playback(&self, user_id: UserId, meditation_id: i64, state: PlaybackState) {
        self.playback
            .lock()
            .unwrap()
            .insert((user_id.value(), meditation_id), state);
    }
}

/// MeditationRepository fake.
#[derive(Clone)]
pub struct FakeMeditationRepository(pub RepoState);

#[async_trait::async_trait]
impl crate::ports::MeditationRepository for FakeMeditationRepository {
    async fn insert(
        &self,
        name: String,
        content_type: &str,
        size_bytes: i64,
        storage_key: String,
        created_at: DateTime<Utc>,
    ) -> Result<meditation_domain::Meditation, String> {
        let mut rows = self.0.meditations.lock().unwrap();
        let id = RepoState::next_id(&rows, |meditation| meditation.id());
        let meditation = meditation_domain::Meditation::new(
            id,
            &name,
            content_type,
            size_bytes,
            &storage_key,
            created_at,
        )
        .map_err(|error| error.to_string())?;
        rows.push(meditation.clone());
        Ok(meditation)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<meditation_domain::Meditation>, String> {
        Ok(self
            .0
            .meditations
            .lock()
            .unwrap()
            .iter()
            .find(|meditation| meditation.id() == id)
            .cloned())
    }

    async fn list_all(&self) -> Result<Vec<meditation_domain::Meditation>, String> {
        let mut rows = self.0.meditations.lock().unwrap().clone();
        rows.sort_by(|left, right| {
            right
                .created_at()
                .cmp(&left.created_at())
                .then_with(|| right.id().cmp(&left.id()))
        });
        Ok(rows)
    }

    async fn delete(&self, id: i64) -> Result<bool, String> {
        let mut rows = self.0.meditations.lock().unwrap();
        let len = rows.len();
        rows.retain(|meditation| meditation.id() != id);
        Ok(rows.len() != len)
    }
}

/// ProgramRepository fake.
#[derive(Clone)]
pub struct FakeProgramRepository(pub RepoState);

#[async_trait::async_trait]
impl ProgramRepository for FakeProgramRepository {
    async fn insert(
        &self,
        title: String,
        description: Option<String>,
        cover_storage_key: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Program, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let id = RepoState::next_id(&rows, |program| program.id);
        let program = Program {
            id,
            title,
            description,
            cover_storage_key,
            category_id: None,
            created_at,
        };
        rows.push(program.clone());
        Ok(program)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Program>, String> {
        Ok(self
            .0
            .programs
            .lock()
            .unwrap()
            .iter()
            .find(|program| program.id == id)
            .cloned())
    }

    async fn list_all(&self) -> Result<Vec<Program>, String> {
        Ok(self.0.programs.lock().unwrap().clone())
    }

    async fn update(
        &self,
        id: i64,
        title: String,
        description: Option<String>,
    ) -> Result<bool, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let Some(program) = rows.iter_mut().find(|program| program.id == id) else {
            return Ok(false);
        };
        program.title = title;
        program.description = description;
        Ok(true)
    }

    async fn delete(&self, id: i64) -> Result<Option<Program>, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let index = rows.iter().position(|program| program.id == id);
        Ok(index.map(|index| rows.remove(index)))
    }

    async fn set_category(&self, id: i64, category_id: Option<i64>) -> Result<bool, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let Some(program) = rows.iter_mut().find(|program| program.id == id) else {
            return Ok(false);
        };
        program.category_id = category_id;
        Ok(true)
    }

    async fn clear_category(&self, category_id: i64) -> Result<u64, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let mut cleared = 0;
        for program in rows.iter_mut() {
            if program.category_id == Some(category_id) {
                program.category_id = None;
                cleared += 1;
            }
        }
        Ok(cleared)
    }

    async fn remove_members_for_program(&self, program_id: i64) -> Result<u64, String> {
        let mut rows = self.0.memberships.lock().unwrap();
        let len = rows.len();
        rows.retain(|membership| membership.program_id != program_id);
        Ok((len - rows.len()) as u64)
    }
}

/// CategoryRepository fake.
#[derive(Clone)]
pub struct FakeCategoryRepository(pub RepoState);

#[async_trait::async_trait]
impl CategoryRepository for FakeCategoryRepository {
    async fn insert(&self, name: String, created_at: DateTime<Utc>) -> Result<Category, String> {
        let mut rows = self.0.categories.lock().unwrap();
        let id = RepoState::next_id(&rows, |category| category.id);
        let category = Category {
            id,
            name,
            created_at,
        };
        rows.push(category.clone());
        Ok(category)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Category>, String> {
        Ok(self
            .0
            .categories
            .lock()
            .unwrap()
            .iter()
            .find(|category| category.name == name)
            .cloned())
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Category>, String> {
        Ok(self
            .0
            .categories
            .lock()
            .unwrap()
            .iter()
            .find(|category| category.id == id)
            .cloned())
    }

    async fn list_all(&self) -> Result<Vec<Category>, String> {
        Ok(self.0.categories.lock().unwrap().clone())
    }

    async fn delete(&self, id: i64) -> Result<bool, String> {
        let mut rows = self.0.categories.lock().unwrap();
        let len = rows.len();
        rows.retain(|category| category.id != id);
        Ok(rows.len() != len)
    }
}

/// MembershipRepository fake.
#[derive(Clone)]
pub struct FakeMembershipRepository(pub RepoState);

#[async_trait::async_trait]
impl MembershipRepository for FakeMembershipRepository {
    async fn add(&self, program_id: i64, meditation_id: i64, added_at: DateTime<Utc>) -> i64 {
        let mut rows = self.0.memberships.lock().unwrap();
        if let Some(existing) = rows.iter().find(|membership| {
            membership.program_id == program_id && membership.meditation_id == meditation_id
        }) {
            return existing.meditation_id;
        }
        let row_id = rows.len() as i64 + 1;
        rows.push(Membership {
            program_id,
            meditation_id,
            added_at,
        });
        row_id
    }

    async fn remove(&self, program_id: i64, meditation_id: i64) -> Result<bool, String> {
        let mut rows = self.0.memberships.lock().unwrap();
        let len = rows.len();
        rows.retain(|membership| {
            !(membership.program_id == program_id && membership.meditation_id == meditation_id)
        });
        Ok(rows.len() != len)
    }

    async fn list_by_program(&self, program_id: i64) -> Result<Vec<Membership>, String> {
        let mut rows: Vec<Membership> = self
            .0
            .memberships
            .lock()
            .unwrap()
            .iter()
            .filter(|membership| membership.program_id == program_id)
            .cloned()
            .collect();
        rows.sort_by_key(|membership| membership.added_at);
        Ok(rows)
    }

    async fn remove_for_meditation(&self, meditation_id: i64) -> Result<u64, String> {
        let mut rows = self.0.memberships.lock().unwrap();
        let len = rows.len();
        rows.retain(|membership| membership.meditation_id != meditation_id);
        Ok((len - rows.len()) as u64)
    }
}

/// SessionRepository fake honoring the (user, meditation, started_at) key.
#[derive(Clone)]
pub struct FakeSessionRepository(pub RepoState);

#[async_trait::async_trait]
impl SessionRepository for FakeSessionRepository {
    async fn record_if_new(
        &self,
        user_id: UserId,
        meditation_id: i64,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
    ) -> Result<MeditationSession, String> {
        let mut rows = self.0.sessions.lock().unwrap();
        if let Some(existing) = rows.iter().find(|session| {
            session.user_id == user_id
                && session.meditation_id == meditation_id
                && session.started_at == started_at
        }) {
            return Ok(existing.clone());
        }
        let id = RepoState::next_id(&rows, |session| session.id);
        let session = MeditationSession {
            id,
            user_id,
            meditation_id,
            started_at,
            finished_at,
        };
        rows.push(session.clone());
        Ok(session)
    }

    async fn count_for(&self, user_id: UserId, meditation_id: i64) -> Result<i64, String> {
        Ok(self
            .0
            .sessions
            .lock()
            .unwrap()
            .iter()
            .filter(|session| session.user_id == user_id && session.meditation_id == meditation_id)
            .count() as i64)
    }
}

/// NoteRepository fake with author scoping.
#[derive(Clone)]
pub struct FakeNoteRepository(pub RepoState);

#[async_trait::async_trait]
impl NoteRepository for FakeNoteRepository {
    async fn insert(
        &self,
        user_id: UserId,
        meditation_id: i64,
        text: String,
        created_at: DateTime<Utc>,
    ) -> Result<Note, String> {
        let mut rows = self.0.notes.lock().unwrap();
        let id = RepoState::next_id(&rows, |note| note.id);
        let note = Note {
            id,
            user_id,
            meditation_id,
            text,
            created_at,
            updated_at: created_at,
        };
        rows.push(note.clone());
        Ok(note)
    }

    async fn list_for(&self, user_id: UserId, meditation_id: i64) -> Result<Vec<Note>, String> {
        let mut rows: Vec<Note> = self
            .0
            .notes
            .lock()
            .unwrap()
            .iter()
            .filter(|note| note.user_id == user_id && note.meditation_id == meditation_id)
            .cloned()
            .collect();
        rows.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(rows)
    }

    async fn update(
        &self,
        note_id: i64,
        user_id: UserId,
        text: String,
        updated_at: DateTime<Utc>,
    ) -> Result<bool, String> {
        let mut rows = self.0.notes.lock().unwrap();
        let Some(note) = rows
            .iter_mut()
            .find(|note| note.id == note_id && note.user_id == user_id)
        else {
            return Ok(false);
        };
        note.text = text;
        note.updated_at = updated_at;
        Ok(true)
    }

    async fn delete(&self, note_id: i64, user_id: UserId) -> Result<bool, String> {
        let mut rows = self.0.notes.lock().unwrap();
        let len = rows.len();
        rows.retain(|note| !(note.id == note_id && note.user_id == user_id));
        Ok(rows.len() != len)
    }

    async fn delete_for_meditation(&self, meditation_id: i64) -> Result<u64, String> {
        let mut rows = self.0.notes.lock().unwrap();
        let len = rows.len();
        rows.retain(|note| note.meditation_id != meditation_id);
        Ok((len - rows.len()) as u64)
    }
}

/// PlaybackStateRepository fake.
#[derive(Clone)]
pub struct FakePlaybackRepository(pub RepoState);

#[async_trait::async_trait]
impl PlaybackStateRepository for FakePlaybackRepository {
    async fn upsert_position(
        &self,
        user_id: UserId,
        meditation_id: i64,
        position_ms: i64,
    ) -> Result<(), String> {
        let mut rows = self.0.playback.lock().unwrap();
        let state = rows
            .entry((user_id.value(), meditation_id))
            .or_insert(PlaybackState {
                position_ms: 0,
                completed: false,
            });
        state.position_ms = position_ms;
        Ok(())
    }

    async fn set_completed(
        &self,
        user_id: UserId,
        meditation_id: i64,
        completed: bool,
    ) -> Result<(), String> {
        let mut rows = self.0.playback.lock().unwrap();
        let state = rows
            .entry((user_id.value(), meditation_id))
            .or_insert(PlaybackState {
                position_ms: 0,
                completed: false,
            });
        state.completed = completed;
        Ok(())
    }

    async fn reset_position(&self, user_id: UserId, meditation_id: i64) -> Result<(), String> {
        let mut rows = self.0.playback.lock().unwrap();
        if let Some(state) = rows.get_mut(&(user_id.value(), meditation_id)) {
            state.position_ms = 0;
        }
        Ok(())
    }

    async fn get(
        &self,
        user_id: UserId,
        meditation_id: i64,
    ) -> Result<Option<PlaybackState>, String> {
        Ok(self
            .0
            .playback
            .lock()
            .unwrap()
            .get(&(user_id.value(), meditation_id))
            .copied())
    }
}
