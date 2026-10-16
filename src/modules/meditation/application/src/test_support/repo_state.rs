use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use kernel::UserId;

use crate::ports::{Category, MeditationSession, Membership, Note, PlaybackState, Program};

/// (user, meditation, touched-at) rows tracking playback touch order.
type PlaybackTouches = Vec<(i64, i64, DateTime<Utc>)>;

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
    /// Touch order for playback rows, oldest first; mirrors the real
    /// repository's `updated_at` ordering for `list_touched`.
    pub playback_touches: Arc<Mutex<PlaybackTouches>>,
    /// When set, `FakeMembershipRepository::add` fails instead of inserting;
    /// exercises the compensation path in commands that add a membership
    /// after already committing other state (e.g. `upload_meditation`).
    pub membership_add_failures: Arc<Mutex<bool>>,
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
