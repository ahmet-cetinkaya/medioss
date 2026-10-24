use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use meditation_application::ports::{MeditationRepository, NoteRepository, SessionRepository};
use meditation_infrastructure::persistence::{
    SeaOrmMeditationRepository, SeaOrmNoteRepository, SeaOrmSessionRepository,
};
use mood_tracker_application::commands::create_mood_entry::CreateMoodEntryService;
use mood_tracker_application::commands::delete_mood_entry::DeleteMoodEntryService;
use mood_tracker_application::commands::update_mood_entry::UpdateMoodEntryService;
use mood_tracker_application::ports::{
    Clock, MeditationTimelineEvent, MeditationTimelineEventKind, MeditationTimelineSource,
    MoodEntryRepository,
};
use mood_tracker_application::queries::get_mood_history::MoodHistoryService;
use mood_tracker_application::queries::get_mood_statistics::GetMoodStatisticsService;
use mood_tracker_application::queries::get_timeline::GetTimelineService;
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
            // Notes are swept when a meditation is deleted, but keep the
            // timeline resilient if an orphan ever appears.
            let Some(meditation) = self.meditations.find_by_id(note.meditation_id).await? else {
                tracing::warn!(
                    note_id = note.id,
                    meditation_id = note.meditation_id,
                    "skipping timeline note whose meditation is missing"
                );
                continue;
            };
            events.push(MeditationTimelineEvent {
                id: note.id,
                meditation_name: meditation.name().to_owned(),
                note: Some(note.text),
                recorded_at: note.created_at,
                kind: MeditationTimelineEventKind::Note,
            });
        }
        for session in self.sessions.list_for_user(user_id).await? {
            // Deleting a meditation does not sweep completed sessions, so
            // orphaned session rows would otherwise break the whole timeline.
            let Some(meditation) = self.meditations.find_by_id(session.meditation_id).await? else {
                tracing::warn!(
                    session_id = session.id,
                    meditation_id = session.meditation_id,
                    "skipping timeline session whose meditation is missing"
                );
                continue;
            };
            events.push(MeditationTimelineEvent {
                id: session.id,
                meditation_name: meditation.name().to_owned(),
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
    mood_tracker_dependencies_with_entries(
        Arc::new(SeaOrmMoodEntryRepository::new(db.clone())),
        db,
        medioss_origin,
    )
}

pub fn mood_tracker_dependencies_with_entries(
    entries: Arc<dyn MoodEntryRepository>,
    db: DatabaseConnection,
    medioss_origin: Arc<String>,
) -> SharedDependencies {
    let meditation: Arc<dyn MeditationTimelineSource> = Arc::new(MeditationTimelineAdapter {
        meditations: SeaOrmMeditationRepository::new(db.clone()),
        notes: SeaOrmNoteRepository::new(db.clone()),
        sessions: SeaOrmSessionRepository::new(db),
    });
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    Arc::new(MoodTrackerDependencies {
        create: Arc::new(CreateMoodEntryService::new(entries.clone(), clock.clone())),
        update: Arc::new(UpdateMoodEntryService::new(entries.clone(), clock.clone())),
        delete: Arc::new(DeleteMoodEntryService::new(entries.clone())),
        list_mood_entries: Arc::new(MoodHistoryService::new(entries.clone())),
        mood_statistics: Arc::new(GetMoodStatisticsService::new(entries.clone())),
        timeline: Arc::new(GetTimelineService::new(entries, meditation)),
        clock_now: clock,
        medioss_origin,
    })
}
