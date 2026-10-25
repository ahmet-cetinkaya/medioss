use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use identity_application::ports::UserRepository;
use kernel::UserId;
use meditation_application::ports::{
    MeditationRepository, NoteRepository, PlaybackStateRepository, SessionRepository,
};
use meditation_domain::meditation::Meditation;
use meditation_infrastructure::persistence::{
    SeaOrmMeditationRepository, SeaOrmNoteRepository, SeaOrmPlaybackRepository,
    SeaOrmSessionRepository,
};
use mood_tracker_application::ports::MoodEntryRepository;
use mood_tracker_infrastructure::persistence::SeaOrmMoodEntryRepository;
use portability_application::models::mood_record::MoodRecord;
use portability_application::models::note_record::NoteRecord;
use portability_application::models::playback_record::PlaybackRecord;
use portability_application::models::practice_snapshot::PracticeSnapshot;
use portability_application::models::session_record::SessionRecord;
use portability_application::ports::meditation_lookup::{MeditationLookup, MeditationRef};
use portability_application::ports::mood_record_sink::MoodRecordSink;
use portability_application::ports::own_mood_records::OwnMoodRecords;
use portability_application::ports::own_practice_records::OwnPracticeRecords;
use portability_application::ports::practice_sink::PracticeSink;
use portability_presentation::SharedDependencies;
use sea_orm::DatabaseConnection;

struct MoodRecords {
    entries: SeaOrmMoodEntryRepository,
}

#[async_trait]
impl OwnMoodRecords for MoodRecords {
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<MoodRecord>, String> {
        Ok(self
            .entries
            .list_all_for_user(user_id)
            .await?
            .into_iter()
            .map(|e| MoodRecord {
                score: e.score().value(),
                note: e.note().map(str::to_owned),
                recorded_at: e.recorded_at(),
            })
            .collect())
    }
}

#[async_trait]
impl MoodRecordSink for MoodRecords {
    async fn insert(&self, user_id: UserId, record: MoodRecord) -> Result<(), String> {
        let score = mood_tracker_domain::MoodScore::new(record.score)
            .map_err(|e| format!("imported score invalid: {e:?}"))?;
        self.entries
            .insert(user_id, score, record.note, record.recorded_at)
            .await
            .map(|_| ())
    }
}

struct PracticeRecords {
    meditations: SeaOrmMeditationRepository,
    sessions: SeaOrmSessionRepository,
    notes: SeaOrmNoteRepository,
    playback: SeaOrmPlaybackRepository,
}

fn reference(meditation: &Meditation) -> (String, i64) {
    (meditation.name().to_owned(), meditation.duration_ms())
}

#[async_trait]
impl OwnPracticeRecords for PracticeRecords {
    async fn list_for_user(&self, user_id: UserId) -> Result<PracticeSnapshot, String> {
        let meditations = self.meditations.list_all().await?;
        let lookup = |id: i64| -> Option<(String, i64)> {
            meditations.iter().find(|m| m.id() == id).map(reference)
        };
        let mut sessions = Vec::new();
        for s in self.sessions.list_for_user(user_id).await? {
            if let Some((name, duration_ms)) = lookup(s.meditation_id) {
                sessions.push(SessionRecord {
                    meditation_name: name,
                    meditation_duration_ms: duration_ms,
                    started_at: s.started_at,
                    finished_at: s.finished_at,
                });
            }
        }
        let mut notes = Vec::new();
        for n in self.notes.list_for_user(user_id).await? {
            if let Some((name, duration_ms)) = lookup(n.meditation_id) {
                notes.push(NoteRecord {
                    meditation_name: name,
                    meditation_duration_ms: duration_ms,
                    text: n.text,
                    created_at: n.created_at,
                });
            }
        }
        let mut playback = Vec::new();
        for m in &meditations {
            if let Some(state) = self.playback.get(user_id, m.id()).await? {
                playback.push(PlaybackRecord {
                    meditation_name: m.name().to_owned(),
                    meditation_duration_ms: m.duration_ms(),
                    position_ms: state.position_ms,
                    completed: state.completed,
                });
            }
        }
        Ok(PracticeSnapshot {
            sessions,
            notes,
            playback,
        })
    }
}

#[async_trait]
impl PracticeSink for PracticeRecords {
    async fn record_session(
        &self,
        user_id: UserId,
        meditation_id: i64,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
    ) -> Result<(), String> {
        self.sessions
            .record_if_new(user_id, meditation_id, started_at, finished_at)
            .await
            .map(|_| ())
    }
    async fn insert_note(
        &self,
        user_id: UserId,
        meditation_id: i64,
        text: &str,
        created_at: DateTime<Utc>,
    ) -> Result<(), String> {
        self.notes
            .insert(user_id, meditation_id, text.to_string(), created_at)
            .await
            .map(|_| ())
    }
    async fn upsert_position(
        &self,
        user_id: UserId,
        meditation_id: i64,
        position_ms: i64,
    ) -> Result<(), String> {
        self.playback
            .upsert_position(user_id, meditation_id, position_ms)
            .await
    }
    async fn set_completed(
        &self,
        user_id: UserId,
        meditation_id: i64,
        completed: bool,
    ) -> Result<(), String> {
        self.playback
            .set_completed(user_id, meditation_id, completed)
            .await
    }
}

struct ProfileNames {
    users: identity_infrastructure::SeaOrmUserRepository,
}

#[async_trait]
impl portability_application::ports::display_name_resolver::DisplayNameResolver for ProfileNames {
    async fn display_name(&self, user_id: UserId) -> Result<Option<String>, String> {
        let user = self
            .users
            .find_by_id(user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("user vanished")?;
        Ok(user.display_name().map(|d| d.as_str().to_owned()))
    }
}

struct Library {
    meditations: SeaOrmMeditationRepository,
}

#[async_trait]
impl MeditationLookup for Library {
    async fn list_all(&self) -> Result<Vec<MeditationRef>, String> {
        Ok(self
            .meditations
            .list_all()
            .await?
            .iter()
            .map(|m| MeditationRef {
                id: m.id(),
                name: m.name().to_owned(),
                duration_ms: m.duration_ms(),
            })
            .collect())
    }
}

pub fn portability_dependencies(
    db: DatabaseConnection,
    medioss_origin: Arc<String>,
) -> SharedDependencies {
    let mood_records = || {
        Arc::new(MoodRecords {
            entries: SeaOrmMoodEntryRepository::new(db.clone()),
        }) as Arc<dyn OwnMoodRecords>
    };
    let mood_sink = || {
        Arc::new(MoodRecords {
            entries: SeaOrmMoodEntryRepository::new(db.clone()),
        }) as Arc<dyn MoodRecordSink>
    };
    let practice_source = || {
        Arc::new(PracticeRecords {
            meditations: SeaOrmMeditationRepository::new(db.clone()),
            sessions: SeaOrmSessionRepository::new(db.clone()),
            notes: SeaOrmNoteRepository::new(db.clone()),
            playback: SeaOrmPlaybackRepository::new(db.clone()),
        }) as Arc<dyn OwnPracticeRecords>
    };
    let practice_sink = || {
        Arc::new(PracticeRecords {
            meditations: SeaOrmMeditationRepository::new(db.clone()),
            sessions: SeaOrmSessionRepository::new(db.clone()),
            notes: SeaOrmNoteRepository::new(db.clone()),
            playback: SeaOrmPlaybackRepository::new(db.clone()),
        }) as Arc<dyn PracticeSink>
    };

    let export_own_data =
        portability_application::queries::export_own_data::ExportOwnDataQueryHandler::new(
            mood_records(),
            practice_source(),
            Arc::new(ProfileNames {
                users: identity_infrastructure::SeaOrmUserRepository::new(db.clone()),
            }),
        );
    let export_mood_records =
        portability_application::queries::export_mood_records::ExportMoodRecordsQueryHandler::new(
            mood_records(),
        );
    let import_own_data =
        portability_application::commands::import_own_data::ImportOwnDataCommandHandler::new(
            mood_records(),
            mood_sink(),
            practice_source(),
            practice_sink(),
            Arc::new(Library {
                meditations: SeaOrmMeditationRepository::new(db.clone()),
            }),
        );

    SharedDependencies {
        export_own_data: Arc::new(export_own_data),
        export_mood_records: Arc::new(export_mood_records),
        import_own_data: Arc::new(import_own_data),
        medioss_origin,
    }
}

#[cfg(test)]
#[path = "portability.tests.rs"]
mod tests;
