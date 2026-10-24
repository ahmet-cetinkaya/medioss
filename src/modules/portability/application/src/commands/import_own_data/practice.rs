use super::meditation_index::MeditationIndex;
use crate::models::import_report::ImportReport;
use crate::models::practice_snapshot::PracticeSnapshot;
use crate::ports::practice_sink::PracticeSink;
use kernel::UserId;
use std::collections::HashSet;

/// What to do with one imported practice record.
enum Disposition {
    Apply(i64),
    Duplicate,
    Unlinked,
}

fn disposition(meditation_id: Option<i64>, is_duplicate: bool) -> Disposition {
    match meditation_id {
        Some(id) if !is_duplicate => Disposition::Apply(id),
        Some(_) => Disposition::Duplicate,
        None => Disposition::Unlinked,
    }
}

fn tally(disposition: &Disposition, report: &mut ImportReport) {
    match disposition {
        Disposition::Apply(_) => report.practice_linked += 1,
        Disposition::Duplicate => report.practice_duplicates_skipped += 1,
        Disposition::Unlinked => report.practice_skipped += 1,
    }
}

/// Decides one incoming record, counts it in the report and returns the
/// meditation to write it against, or `None` when it must be skipped.
fn classify<K: Eq + std::hash::Hash>(
    existing_keys: &HashSet<K>,
    key: &K,
    linked_meditation: Option<i64>,
    report: &mut ImportReport,
) -> Option<i64> {
    let outcome = disposition(linked_meditation, existing_keys.contains(key));
    tally(&outcome, report);
    match outcome {
        Disposition::Apply(id) => Some(id),
        Disposition::Duplicate | Disposition::Unlinked => None,
    }
}

pub(super) async fn apply_practice(
    user_id: UserId,
    incoming: PracticeSnapshot,
    existing: &PracticeSnapshot,
    index: &MeditationIndex<'_>,
    sink: &dyn PracticeSink,
    report: &mut ImportReport,
) -> Result<(), String> {
    let session_keys: HashSet<_> = existing
        .sessions
        .iter()
        .map(|s| {
            (
                s.meditation_name.as_str(),
                s.meditation_duration_ms,
                s.started_at,
            )
        })
        .collect();
    let note_keys: HashSet<_> = existing
        .notes
        .iter()
        .map(|n| {
            (
                n.meditation_name.as_str(),
                n.meditation_duration_ms,
                n.text.as_str(),
                n.created_at,
            )
        })
        .collect();
    let playback_keys: HashSet<_> = existing
        .playback
        .iter()
        .map(|p| {
            (
                p.meditation_name.as_str(),
                p.meditation_duration_ms,
                p.position_ms,
                p.completed,
            )
        })
        .collect();
    let link = |name: &str, duration_ms: i64| index.get(&(name, duration_ms)).copied();

    for session in incoming.sessions {
        let key = (
            session.meditation_name.as_str(),
            session.meditation_duration_ms,
            session.started_at,
        );
        let linked = link(&session.meditation_name, session.meditation_duration_ms);
        if let Some(id) = classify(&session_keys, &key, linked, report) {
            sink.record_session(user_id, id, session.started_at, session.finished_at)
                .await?;
        }
    }
    for note in incoming.notes {
        let key = (
            note.meditation_name.as_str(),
            note.meditation_duration_ms,
            note.text.as_str(),
            note.created_at,
        );
        let linked = link(&note.meditation_name, note.meditation_duration_ms);
        if let Some(id) = classify(&note_keys, &key, linked, report) {
            sink.insert_note(user_id, id, &note.text, note.created_at)
                .await?;
        }
    }
    for playback in incoming.playback {
        let key = (
            playback.meditation_name.as_str(),
            playback.meditation_duration_ms,
            playback.position_ms,
            playback.completed,
        );
        let linked = link(&playback.meditation_name, playback.meditation_duration_ms);
        if let Some(id) = classify(&playback_keys, &key, linked, report) {
            sink.upsert_position(user_id, id, playback.position_ms)
                .await?;
            if playback.completed {
                sink.set_completed(user_id, id, true).await?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "practice.tests.rs"]
mod tests;
