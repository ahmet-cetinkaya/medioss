use super::note_record::NoteRecord;
use super::playback_record::PlaybackRecord;
use super::session_record::SessionRecord;

#[derive(Debug, Clone, Default)]
pub struct PracticeSnapshot {
    pub sessions: Vec<SessionRecord>,
    pub notes: Vec<NoteRecord>,
    pub playback: Vec<PlaybackRecord>,
}
