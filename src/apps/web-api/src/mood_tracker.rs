use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use meditation_application::ports::{MeditationRepository, NoteRepository, SessionRepository};
use meditation_infrastructure::persistence::{
    SeaOrmMeditationRepository, SeaOrmNoteRepository, SeaOrmSessionRepository,
};
use mood_tracker_application::ports::{
    Clock, MeditationTimelineEvent, MeditationTimelineEventKind, MeditationTimelineSource,
};
use mood_tracker_infrastructure::persistence::SeaOrmMoodEntryRepository;
use mood_tracker_presentation::{MoodTrackerDependencies, SharedDependencies};
use sea_orm::DatabaseConnection;

pub struct SystemClock;

#[async_trait]
impl Clock for SystemClock {
    async fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

struct MeditationTimelineAdapter {
    meditations: SeaOrmMeditationRepository,
    notes: SeaOrmNoteRepository,
    sessions: SeaOrmSessionRepository,
}

#[async_trait]
impl MeditationTimelineSource for MeditationTimelineAdapter {
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<MeditationTimelineEvent>, String> {
        let mut events = Vec::new();
        for note in self.notes.list_for_user(user_id).await? {
            let name = self
                .meditations
                .find_by_id(note.meditation_id)
                .await?
                .ok_or_else(|| "meditation missing for note".to_owned())?
                .name()
                .to_owned();
            events.push(MeditationTimelineEvent {
                id: note.id,
                meditation_name: name,
                note: Some(note.text),
                recorded_at: note.created_at,
                kind: MeditationTimelineEventKind::Note,
            });
        }
        for session in self.sessions.list_for_user(user_id).await? {
            let name = self
                .meditations
                .find_by_id(session.meditation_id)
                .await?
                .ok_or_else(|| "meditation missing for session".to_owned())?
                .name()
                .to_owned();
            events.push(MeditationTimelineEvent {
                id: session.id,
                meditation_name: name,
                note: None,
                recorded_at: session.finished_at,
                kind: MeditationTimelineEventKind::CompletedSession,
            });
        }
        Ok(events)
    }
}

pub fn mood_tracker_dependencies(
    db: DatabaseConnection,
    medioss_origin: Arc<String>,
) -> SharedDependencies {
    Arc::new(MoodTrackerDependencies {
        entries: Arc::new(SeaOrmMoodEntryRepository::new(db.clone())),
        meditation: Arc::new(MeditationTimelineAdapter {
            meditations: SeaOrmMeditationRepository::new(db.clone()),
            notes: SeaOrmNoteRepository::new(db.clone()),
            sessions: SeaOrmSessionRepository::new(db),
        }),
        clock: Arc::new(SystemClock),
        medioss_origin,
    })
}
