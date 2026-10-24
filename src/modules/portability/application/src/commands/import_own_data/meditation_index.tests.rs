use super::*;

fn reference(id: i64, name: &str, duration_ms: i64) -> MeditationRef {
    MeditationRef {
        id,
        name: name.into(),
        duration_ms,
    }
}

#[test]
fn empty_library_gives_empty_index() {
    assert!(meditation_index(&[]).is_empty());
}

#[test]
fn indexes_by_name_and_duration() {
    let library = [reference(1, "Calm", 1000), reference(2, "Calm", 2000)];
    let index = meditation_index(&library);
    assert_eq!(index.len(), 2);
    assert_eq!(index[&("Calm", 1000)], 1);
    assert_eq!(index[&("Calm", 2000)], 2);
}

#[test]
fn duplicate_pair_keeps_lowest_id_regardless_of_order() {
    let library = [
        reference(9, "Calm", 1000),
        reference(4, "Calm", 1000),
        reference(6, "Calm", 1000),
    ];
    let index = meditation_index(&library);
    assert_eq!(index.len(), 1);
    assert_eq!(index[&("Calm", 1000)], 4);
}

#[test]
fn name_match_is_case_sensitive() {
    let library = [reference(1, "Calm", 1000)];
    assert!(!meditation_index(&library).contains_key(&("calm", 1000)));
}
