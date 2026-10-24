use super::*;

fn item(sound_id: i64, volume: i64, position: i64) -> MixItem {
    MixItem {
        sound_id,
        volume,
        position,
    }
}

fn invalid() -> Result<(), ApplicationError> {
    Err(ApplicationError::InvalidInput("mix items"))
}

#[test]
fn well_formed_items_pass_including_volume_bounds() {
    let items = [item(1, 0, 0), item(2, 100, 1)];
    assert_eq!(validate_mix_items(&items), Ok(()));
}

#[test]
fn empty_items_are_rejected() {
    assert_eq!(validate_mix_items(&[]), invalid());
}

#[test]
fn item_count_is_bounded() {
    let at_limit: Vec<_> = (0..MAX_MIX_SOUNDS as i64)
        .map(|n| item(n + 1, 50, n))
        .collect();
    assert_eq!(validate_mix_items(&at_limit), Ok(()));

    let over: Vec<_> = (0..=MAX_MIX_SOUNDS as i64)
        .map(|n| item(n + 1, 50, n))
        .collect();
    assert_eq!(validate_mix_items(&over), invalid());
}

#[test]
fn non_positive_sound_ids_are_rejected() {
    assert_eq!(validate_mix_items(&[item(0, 50, 0)]), invalid());
    assert_eq!(validate_mix_items(&[item(-1, 50, 0)]), invalid());
}

#[test]
fn duplicate_sounds_are_rejected() {
    assert_eq!(
        validate_mix_items(&[item(1, 50, 0), item(1, 50, 1)]),
        invalid()
    );
}

#[test]
fn out_of_range_volumes_are_rejected() {
    assert_eq!(validate_mix_items(&[item(1, -1, 0)]), invalid());
    assert_eq!(validate_mix_items(&[item(1, 101, 0)]), invalid());
}

#[test]
fn positions_must_be_contiguous_from_zero() {
    assert_eq!(validate_mix_items(&[item(1, 50, 1)]), invalid());
    assert_eq!(
        validate_mix_items(&[item(1, 50, 0), item(2, 50, 2)]),
        invalid()
    );
}
