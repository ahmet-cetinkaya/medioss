use crate::ports::meditation_lookup::MeditationRef;
use std::collections::HashMap;

/// Maps (name, duration) to the lowest meditation id sharing that pair.
pub(super) type MeditationIndex<'a> = HashMap<(&'a str, i64), i64>;

pub(super) fn meditation_index(meditations: &[MeditationRef]) -> MeditationIndex<'_> {
    let mut index = HashMap::new();
    for m in meditations {
        index
            .entry((m.name.as_str(), m.duration_ms))
            .and_modify(|current: &mut i64| *current = (*current).min(m.id))
            .or_insert(m.id);
    }
    index
}
