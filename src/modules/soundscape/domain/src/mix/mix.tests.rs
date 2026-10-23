use super::*;

#[test]
fn mix_invariants_reject_duplicates_and_bad_volume() {
    let now = Utc::now();
    assert!(Mix::new(
        1,
        1,
        "Mix",
        vec![
            MixItem {
                sound_id: 1,
                volume: 100,
                position: 0
            },
            MixItem {
                sound_id: 1,
                volume: 50,
                position: 1
            }
        ],
        now,
        now
    )
    .is_err());
    assert!(Mix::new(
        1,
        1,
        "Mix",
        vec![MixItem {
            sound_id: 1,
            volume: 101,
            position: 0
        }],
        now,
        now
    )
    .is_err());
}

#[test]
fn mix_hydration_tolerates_empty_items_after_sound_deletion() {
    let now = Utc::now();
    let mix = Mix::new(1, 1, "Mix", Vec::new(), now, now).expect("empty mix hydrates");
    assert!(mix.items.is_empty());
}
