use super::MoodScore;

#[test]
fn accepts_the_inclusive_score_boundaries() {
    assert_eq!(MoodScore::new(1).unwrap().value(), 1);
    assert_eq!(MoodScore::new(5).unwrap().value(), 5);
}

#[test]
fn rejects_scores_outside_the_allowed_range() {
    assert!(MoodScore::new(0).is_err());
    assert!(MoodScore::new(6).is_err());
}
